//! Reading a Claude Code stream-json transcript into transcript records, the model, whether the
//! server connected, and the error text of a failed session. Fields are read leniently, since the
//! format is Claude Code's, not ours.

use super::SERVER_NAME;
use crate::records::transcript::{Content, Record};
use serde_json::Value;

/// What a stream says.
#[derive(Debug, Default)]
pub struct Parsed {
    /// The transcript records, in order.
    pub records: Vec<Record>,
    /// The model id the init record names.
    pub model: Option<String>,
    /// False when the init record says the clattertrap server did not connect.
    pub connected: bool,
    /// The final text of a session Claude Code marked as failed.
    pub error: Option<String>,
}

/// Parses stream-json text, skipping lines that are not records.
#[must_use]
pub fn parse(text: &str) -> Parsed {
    let mut out = Parsed {
        connected: true,
        ..Parsed::default()
    };
    for line in text.lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match record["type"].as_str() {
            Some("system") if record["subtype"] == "init" => {
                out.connected = server_connected(&record);
                out.model = record["model"].as_str().map(ToOwned::to_owned);
            }
            Some("assistant") => out.records.extend(assistant_blocks(&record)),
            Some("user") => out.records.extend(tool_results(&record)),
            Some("result") => {
                out.error = failure_text(&record);
                out.records.push(result_record(&record));
            }
            _ => {}
        }
    }
    out
}

fn server_connected(init: &Value) -> bool {
    init["mcp_servers"].as_array().is_some_and(|servers| {
        servers
            .iter()
            .any(|s| s["name"] == SERVER_NAME && s["status"] == "connected")
    })
}

fn blocks(record: &Value) -> &[Value] {
    record["message"]["content"]
        .as_array()
        .map_or(&[], Vec::as_slice)
}

fn assistant_blocks(record: &Value) -> Vec<Record> {
    blocks(record)
        .iter()
        .filter_map(|block| match block["type"].as_str() {
            Some("text") => Some(Record::Text {
                text: block["text"].as_str().unwrap_or("").to_owned(),
            }),
            Some("tool_use") => Some(Record::ToolUse {
                name: block["name"].as_str().unwrap_or("").to_owned(),
                input: block["input"].clone(),
            }),
            _ => None,
        })
        .collect()
}

fn tool_results(record: &Value) -> Vec<Record> {
    blocks(record)
        .iter()
        .filter(|block| block["type"] == "tool_result")
        .map(|block| Record::ToolResult {
            content: contents(&block["content"]),
            is_error: block["is_error"].as_bool().unwrap_or(false),
        })
        .collect()
}

/// A result's content: a list of blocks, or one string Claude Code wrote itself.
fn contents(content: &Value) -> Vec<Content> {
    match content {
        Value::String(text) => vec![Content::Text { text: text.clone() }],
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|block| match block["type"].as_str() {
                Some("text") => Some(Content::Text {
                    text: block["text"].as_str().unwrap_or("").to_owned(),
                }),
                Some("image") => Some(Content::Image { omitted: true }),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

/// The final text, when Claude Code marked the session as failed.
fn failure_text(record: &Value) -> Option<String> {
    let subtype = record["subtype"].as_str().unwrap_or("");
    let failed = record["is_error"].as_bool().unwrap_or(false) || subtype.starts_with("error");
    failed.then(|| record["result"].as_str().unwrap_or("").to_owned())
}

#[allow(clippy::cast_possible_truncation)] // a turn count, never large
fn result_record(record: &Value) -> Record {
    Record::Result {
        subtype: record["subtype"].as_str().unwrap_or("").to_owned(),
        num_turns: record["num_turns"].as_u64().map(|n| n as u32),
        duration_ms: record["duration_ms"].as_u64(),
        total_cost_usd: record["total_cost_usd"].as_f64(),
    }
}
