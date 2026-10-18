//! Reading a Claude Code steam-json transcript: the init record, every tool call, and the final
//! result. Fields are read leniently, since the format is Claude Code's, not ours.

use super::claude::SERVER_NAME;
use crate::records::Status;
use serde_json::Value;

/// The final `result` record.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultRecord {
    /// `success`, `error_max_budget_usd`, `error_during_execution`, and so on.
    pub subtype: String,
    /// Whether Claude Code marked the session as failed.
    pub is_error: bool,
    /// Turns taken.
    pub num_turns: Option<u32>,
    /// Claude Code's cost estimate.
    pub total_cost_usd: Option<f64>,
    /// Wall-clock milliseconds as Claude Code measured them.
    pub duration_ms: Option<u64>,
    /// The final assistant text.
    pub text: Option<String>,
}

/// What a transcript says.
#[derive(Debug, Default)]
pub struct Transcript {
    /// False when the init record says the clattertrap server did not connect.
    pub mcp_connected: bool,
    /// The model id the init record names.
    pub model: Option<String>,
    /// Every tool the contestant called, in order.
    pub tool_calls: Vec<String>,
    /// The final record, when the session produced one.
    pub result: Option<ResultRecord>,
}

impl Transcript {
    /// Parses stream-json text, skipping lines that are not records.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut t = Self {
            mcp_connected: true,
            ..Self::default()
        };
        for line in text.lines() {
            let Ok(record) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            match record["type"].as_str() {
                Some("system") if record["subtype"] == "init" => {
                    t.mcp_connected = server_connected(&record);
                    t.model = record["model"].as_str().map(ToOwned::to_owned);
                }
                Some("assistant") => t.read_tool_uses(&record),
                Some("result") => t.result = Some(result_record(&record)),
                _ => {}
            }
        }
        t
    }

    /// Tools called that are not clattertrap MCP tools, in order, without repeats.
    #[must_use]
    pub fn foreign_tools(&self) -> Vec<String> {
        let prefix = format!("mcp__{SERVER_NAME}__");
        let mut out: Vec<String> = Vec::new();
        for name in &self.tool_calls {
            if !name.starts_with(&prefix) && !out.contains(name) {
                out.push(name.clone());
            }
        }
        out
    }

    fn read_tool_uses(&mut self, record: &Value) {
        let Some(blocks) = record["message"]["content"].as_array() else {
            return;
        };
        for block in blocks {
            if block["type"] == "tool_use"
                && let Some(name) = block["name"].as_str()
            {
                self.tool_calls.push(name.to_owned());
            }
        }
    }
}

fn server_connected(init: &Value) -> bool {
    init["mcp_servers"].as_array().is_some_and(|servers| {
        servers
            .iter()
            .any(|s| s["name"] == SERVER_NAME && s["status"] == "connected")
    })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // counts, never large or negative
fn result_record(record: &Value) -> ResultRecord {
    ResultRecord {
        subtype: record["subtype"].as_str().unwrap_or("").to_owned(),
        is_error: record["is_error"].as_bool().unwrap_or(false),
        num_turns: record["num_turns"].as_u64().map(|n| n as u32),
        total_cost_usd: record["total_cost_usd"].as_f64(),
        duration_ms: record["duration_ms"].as_u64(),
        text: record["result"].as_str().map(ToOwned::to_owned),
    }
}

/// How the run ended, with the reason for an aborted or flagged one.
#[must_use]
pub fn status(t: &Transcript, submitted: bool, timed_out: bool) -> (Status, Option<String>) {
    let foreign = t.foreign_tools();
    if let Some(first) = foreign.first() {
        return (Status::Flagged, Some(format!("called {first}")));
    }
    if submitted {
        return (Status::Submitted, None);
    }
    if timed_out {
        return (Status::TimedOut, Some("the wall clock ran out".to_owned()));
    }
    let Some(result) = &t.result else {
        return (Status::Aborted, Some("no result record".to_owned()));
    };
    if !t.mcp_connected {
        return (
            Status::Aborted,
            Some(format!("the {SERVER_NAME} MCP server did not connect")),
        );
    }
    if result.subtype == "error_max_budget_usd" {
        return (
            Status::OverBudget,
            Some("the dollar cap ran out".to_owned()),
        );
    }
    if result.is_error || result.subtype.starts_with("error") {
        let text = result.text.as_deref().unwrap_or("");
        return (Status::Aborted, Some(format!("{}: {text}", result.subtype)));
    }
    (Status::NoSubmission, None)
}
