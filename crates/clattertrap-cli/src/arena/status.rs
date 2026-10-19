//! How a run ended, read from what any provider left behind.

use super::SERVER_NAME;
use super::provider::Finished;
use crate::mcp::TOOL_NAMES;
use crate::records::Status;
use crate::records::transcript::Record;

/// Whether a tool name is one of ours: a bare tool name, or the name Claude Code gives it,
/// `mcp__clattertrap__<tool>`.
#[must_use]
pub fn is_ours(name: &str) -> bool {
    let prefix = format!("mcp__{SERVER_NAME}__");
    let bare = name.strip_prefix(prefix.as_str()).unwrap_or(name);
    TOOL_NAMES.contains(&bare)
}

/// Tools called that are not ours, in order, without repeats.
#[must_use]
pub fn foreign_tools(records: &[Record]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for record in records {
        if let Record::ToolUse { name, .. } = record
            && !is_ours(name)
            && !out.contains(name)
        {
            out.push(name.clone());
        }
    }
    out
}

/// The last result record.
#[must_use]
pub fn last_result(records: &[Record]) -> Option<&Record> {
    records
        .iter()
        .rev()
        .find(|r| matches!(r, Record::Result { .. }))
}

/// The run's status, with the reason for an aborted or flagged one.
#[must_use]
pub fn status(f: &Finished, submitted: bool) -> (Status, Option<String>) {
    let foreign = foreign_tools(&f.records);
    if let Some(first) = foreign.first() {
        return (Status::Flagged, Some(format!("called {first}")));
    }
    if submitted {
        return (Status::Submitted, None);
    }
    if f.timed_out {
        return (Status::TimedOut, Some("the wall clock ran out".to_owned()));
    }
    let Some(Record::Result { subtype, .. }) = last_result(&f.records) else {
        return (Status::Aborted, Some("no result record".to_owned()));
    };
    if !f.connected {
        return (
            Status::Aborted,
            Some(format!("the {SERVER_NAME} MCP server did not connect")),
        );
    }
    if subtype == "error_max_budget_usd" {
        return (
            Status::OverBudget,
            Some("the dollar cap ran out".to_owned()),
        );
    }
    if subtype.starts_with("error") || f.error.is_some() {
        let text = f.error.as_deref().unwrap_or("");
        return (Status::Aborted, Some(format!("{subtype}: {text}")));
    }
    (Status::NoSubmission, None)
}
