//! The transcript record: what a contestant said and called, without pictures, session
//! bookkeeping, or local paths. The runner writes it for every provider and the gallery keeps it
//! verbatim; docs/format.md lists the shape.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One line of a transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    /// A text block the contestant wrote.
    Text {
        /// The text.
        text: String,
    },
    /// A tool the contestant called.
    ToolUse {
        /// The tool name as the provider saw it.
        name: String,
        /// The call's input.
        input: Value,
    },
    /// What a tool answered.
    ToolResult {
        /// The answer's blocks, pictures replaced by a marker.
        content: Vec<Content>,
        /// Whether the tool refused the call.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
    /// How the session ended.
    Result {
        /// The provider's end state, `success` or an error kind.
        subtype: String,
        /// Turns taken, when reported.
        num_turns: Option<u32>,
        /// Wall-clock milliseconds as the provider measured them, when reported.
        duration_ms: Option<u64>,
        /// The provider's cost estimate, when reported.
        total_cost_usd: Option<f64>,
    },
}

/// One block of a tool's answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    /// Text.
    Text {
        /// The text.
        text: String,
    },
    /// A picture, left out of the transcript.
    Image {
        /// Always true: the marker says a picture was here.
        omitted: bool,
    },
}

/// One compact JSON object per line, with a trailing newline.
#[must_use]
pub fn to_jsonl(records: &[Record]) -> String {
    let mut text = String::new();
    for record in records {
        if let Ok(line) = serde_json::to_string(record) {
            text.push_str(&line);
            text.push('\n');
        }
    }
    text
}

/// The records in a transcript, in order; lines that are not records are skipped.
#[must_use]
pub fn from_jsonl(text: &str) -> Vec<Record> {
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}
