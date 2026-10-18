//! The gallery transcript: what a contestant said and called, exported from the runner's stream
//! without pictures, session bookkeeping, or local paths. The shape is ours; docs/format.md lists it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One line of a gallery transcript.
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
        /// The tool name as the runner recorded it.
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
        /// The runner's end state, `success` or an error kind.
        subtype: String,
        /// Turns taken, when reported.
        num_turns: Option<u32>,
        /// Wall-clock milliseconds as the runner measured them, when reported.
        duration_ms: Option<u64>,
        /// The runner's cost estimate, when reported.
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
    /// A picture, left out of the gallery.
    Image {
        /// Always true: the marker says a picture was here.
        omitted: bool,
    },
}

/// The records of a stream-json transcript, in order; lines that are not records are skipped.
#[must_use]
pub fn export(stream_json: &str) -> Vec<Record> {
    let mut out = Vec::new();
    for line in stream_json.lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match record["type"].as_str() {
            Some("assistant") => out.extend(assistant_blocks(&record)),
            Some("user") => out.extend(tool_results(&record)),
            Some("result") => out.push(result_record(&record)),
            _ => {}
        }
    }
    out
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

/// A result's content: a list of blocks, or one string the runner wrote itself.
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

#[allow(clippy::cast_possible_truncation)] // a turn count, never large
fn result_record(record: &Value) -> Record {
    Record::Result {
        subtype: record["subtype"].as_str().unwrap_or("").to_owned(),
        num_turns: record["num_turns"].as_u64().map(|n| n as u32),
        duration_ms: record["duration_ms"].as_u64(),
        total_cost_usd: record["total_cost_usd"].as_f64(),
    }
}
