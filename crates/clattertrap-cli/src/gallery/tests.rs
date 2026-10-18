//! The transcript export: what a gallery transcript keeps from a contestant session and what it drops.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::transcript::{Content, Record, export, to_jsonl};
use serde_json::json;

const STREAM: &str = concat!(
    r#"{"type":"system","subtype":"init","cwd":"C:\Projects\clattertrap\pilot\runs\x","slash_commands":["clear"],"model":"claude-opus-5-5","mcp_servers":[{"name":"clattertrap","status":"connected"}]}"#,
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
fn the_export_keeps_text_tool_calls_results_and_the_final_record() {
    let records = export(STREAM);
    assert_eq!(
        records,
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
fn the_jsonl_holds_no_images_paths_or_thinking_and_round_trips() {
    let text = to_jsonl(&export(STREAM));
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
    assert!(text.ends_with('\n'));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with(r#"{"type":"text""#), "{}", lines[0]);
    let back: Vec<Record> = lines
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(back, export(STREAM));
    assert!(!lines[2].contains("is_error"), "{}", lines[2]);
    assert!(lines[3].contains(r#""is_error":true"#), "{}", lines[3]);
}

#[test]
fn the_smoke_transcript_exports_its_tool_calls_and_one_image_marker() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/transcripts/haiku-smoke.jsonl"),
    )
    .unwrap();
    let records = export(&text);
    let uses = records
        .iter()
        .filter(|r| matches!(r, Record::ToolUse { .. }))
        .count();
    let results = records
        .iter()
        .filter(|r| matches!(r, Record::ToolResult { .. }))
        .count();
    assert_eq!((uses, results), (2, 2));
    assert_eq!(to_jsonl(&records).matches("\"omitted\"").count(), 1);
    assert!(matches!(records.last(), Some(Record::Result { subtype, .. }) if subtype == "success"));
}
