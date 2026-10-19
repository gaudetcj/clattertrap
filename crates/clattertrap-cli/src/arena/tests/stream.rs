//! Reading Claude Code's stream-json into the gallery record format, the model, and whether the
//! server connected.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::super::stream::parse;
use crate::records::transcript::{Content, Record, to_jsonl};
use serde_json::json;

const SMOKE: &str = include_str!("../../../tests/fixtures/transcripts/haiku-smoke.jsonl");

const STREAM: &str = concat!(
    r#"{"type":"system","subtype":"init","cwd":"C:\\Projects\\clattertrap\\pilot\\runs\\x","slash_commands":["clear"],"model":"claude-opus-5-5","mcp_servers":[{"name":"clattertrap","status":"connected"}]}"#,
    "\n",
    r#"{"type":"system","subtype":"thinking_tokens","tokens":12}"#,
    "\n",
    r#"{"type":"assistant","message":{"model":"claude-opus-5-5","content":[{"type":"thinking","thinking":"","signature":"abc"},{"type":"text","text":"Looking at the level."},{"type":"tool_use","id":"toolu_1","name":"mcp__clattertrap__level","input":{"id":"bell-drop"},"caller":{"type":"direct"}}],"usage":{"input_tokens":2}}}"#,
    "\n",
    r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_1","type":"tool_result","content":[{"type":"text","text":"bell-drop: Ring the bell"},{"type":"image","source":{"type":"base64","media_type":"image/png","data":"iVBORw0KGgo="}}]}]}}"#,
    "\n",
    r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed"}}"#,
    "\n",
    "not json at all\n",
    r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_2","type":"tool_result","content":"prediction: 284 characters is over the 280 allowed; shorten it","is_error":true}]}}"#,
    "\n",
    r#"{"type":"result","subtype":"success","is_error":false,"num_turns":8,"duration_ms":79141,"total_cost_usd":0.24,"result":"The bell rang.","modelUsage":{"claude-opus-5-5":{}},"permission_denials":[]}"#,
    "\n",
);

#[test]
fn the_records_keep_text_tool_calls_results_and_the_final_record() {
    let parsed = parse(STREAM);
    assert!(parsed.connected);
    assert_eq!(parsed.model.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(parsed.error, None);
    assert_eq!(
        parsed.records,
        [
            Record::Text {
                text: "Looking at the level.".to_owned()
            },
            Record::ToolUse {
                name: "mcp__clattertrap__level".to_owned(),
                input: json!({ "id": "bell-drop" }),
            },
            Record::ToolResult {
                content: vec![
                    Content::Text {
                        text: "bell-drop: Ring the bell".to_owned()
                    },
                    Content::Image { omitted: true },
                ],
                is_error: false,
            },
            Record::ToolResult {
                content: vec![Content::Text {
                    text: "prediction: 284 characters is over the 280 allowed; shorten it"
                        .to_owned()
                }],
                is_error: true,
            },
            Record::Result {
                subtype: "success".to_owned(),
                num_turns: Some(8),
                duration_ms: Some(79141),
                total_cost_usd: Some(0.24),
            },
        ]
    );
}

#[test]
fn the_jsonl_holds_no_images_paths_or_thinking() {
    let text = to_jsonl(&parse(STREAM).records);
    for banned in [
        "base64",
        "iVBOR",
        "cwd",
        "C:\\\\",
        "Projects",
        "thinking",
        "slash_commands",
        "modelUsage",
        "The bell rang",
    ] {
        assert!(!text.contains(banned), "{banned} in {text}");
    }
    assert_eq!(
        text.matches(r#"{"type":"image","omitted":true}"#).count(),
        1
    );
    assert_eq!(text.lines().count(), 5);
}

#[test]
fn the_smoke_transcript_parses_into_calls_and_a_result() {
    let parsed = parse(SMOKE);
    assert!(parsed.connected);
    assert_eq!(parsed.model.as_deref(), Some("claude-haiku-5-5"));
    let uses: Vec<&str> = parsed
        .records
        .iter()
        .filter_map(|r| match r {
            Record::ToolUse { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(uses, ["mcp__clattertrap__rules", "mcp__clattertrap__level"]);
    assert_eq!(to_jsonl(&parsed.records).matches("\"omitted\"").count(), 1);
    assert!(matches!(
        parsed.records.last(),
        Some(Record::Result { subtype, num_turns: Some(3), total_cost_usd: Some(c), .. })
            if subtype == "success" && *c > 0.0
    ));
    assert_eq!(parsed.error, None);
}

#[test]
fn a_failed_result_keeps_its_text_as_the_error() {
    let failed = r#"{"type":"result","subtype":"error_during_execution","is_error":true,"num_turns":1,"total_cost_usd":0.1,"result":"API Error: 529 overloaded"}"#;
    let parsed = parse(failed);
    assert_eq!(parsed.error.as_deref(), Some("API Error: 529 overloaded"));
    assert!(
        matches!(parsed.records.as_slice(), [Record::Result { subtype, .. }] if subtype == "error_during_execution")
    );
}

#[test]
fn a_server_that_never_connected_is_read_from_the_init_record() {
    let init = r#"{"type":"system","subtype":"init","tools":[],"mcp_servers":[{"name":"clattertrap","status":"failed"}]}
{"type":"result","subtype":"success","is_error":false,"num_turns":1,"total_cost_usd":0.01,"result":"I have no tools."}"#;
    assert!(!parse(init).connected);
    assert!(parse("").connected);
}

#[test]
fn junk_lines_are_skipped() {
    let text = "not json\n{\"type\":\"rate_limit_event\"}\n\n{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"x\"}\n";
    let parsed = parse(text);
    assert_eq!(parsed.records.len(), 1);
    assert!(matches!(parsed.records[0], Record::Result { .. }));
}
