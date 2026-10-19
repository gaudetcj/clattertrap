//! The record types read and written by the session, the runner, the report, promote, and verify.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{Call, Status, Submission, Summary, Tool, call_path, read_calls, scene_json};
use crate::mcp::arm::Arm;
use clattertrap_core::score::Outcome;
use serde_json::json;
use std::path::Path;

fn fixture(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("runs")
        .join(rel)
}

#[test]
fn a_rejected_call_round_trips_without_an_outcome() {
    let call = Call {
        seq: 3,
        tool: Tool::Run,
        scene: json!({ "level": "bell-drop", "parts": [] }),
        problems: Some(vec!["parts[0] (beam b): overlaps floor".to_owned()]),
        outcome: None,
        name: None,
        prediction: None,
    };
    let text = serde_json::to_string_pretty(&call).unwrap();
    assert!(text.contains("\"tool\": \"run\""), "{text}");
    assert!(!text.contains("outcome"), "{text}");
    assert!(!text.contains("\"name\""), "{text}");
    let back: Call = serde_json::from_str(&text).unwrap();
    assert_eq!(back, call);
    assert!(!back.accepted());
}

#[test]
fn an_accepted_submit_call_keeps_its_outcome_name_and_prediction() {
    let path = fixture("claude-haiku-5-5/bell-drop/workshop-01/calls/0005-submit.json");
    let call: Call = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(call.seq, 5);
    assert_eq!(call.tool, Tool::Submit);
    assert!(call.accepted());
    let outcome = call.outcome.as_ref().unwrap();
    assert!(!outcome.goal_met);
    assert_eq!(
        serde_json::to_value(outcome).unwrap()["state_hash"],
        "0000000000000002"
    );
    assert_eq!(call.name, None);
}

#[test]
fn the_pilot_fixtures_parse_as_typed_records() {
    let submission: Submission = serde_json::from_str(
        &std::fs::read_to_string(fixture(
            "claude-haiku-5-5/bell-drop/workshop-01/submission.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(submission.level, "bell-drop");
    assert_eq!(submission.arm, Arm::Workshop);
    assert_eq!(submission.name, "Two planks");
    assert!(submission.outcome.goal_met);
    let summary: Summary = serde_json::from_str(
        &std::fs::read_to_string(fixture(
            "claude-opus-5-5/grand-tour/workshop-01/summary.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(summary.status, Status::Flagged);
    assert_eq!(summary.foreign_tools, ["Bash"]);
    assert!(!summary.status.counts());
}

#[test]
fn calls_are_read_in_name_order_and_a_broken_file_stays_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("calls");
    std::fs::create_dir(&calls).unwrap();
    for (seq, tool) in [(2, Tool::Run), (1, Tool::Check), (10, Tool::Submit)] {
        let call = Call {
            seq,
            tool,
            scene: json!({ "level": "x", "parts": [] }),
            problems: None,
            outcome: None,
            name: None,
            prediction: None,
        };
        super::write_call(dir.path(), &call).unwrap();
    }
    std::fs::write(calls.join("0003-run.json"), "{ not json").unwrap();
    std::fs::write(calls.join("notes.txt"), "ignored").unwrap();
    let read = read_calls(dir.path()).unwrap();
    let names: Vec<String> = read
        .iter()
        .map(|(p, _)| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "0001-check.json",
            "0002-run.json",
            "0003-run.json",
            "0010-submit.json"
        ]
    );
    assert!(read[2].1.is_err());
    assert_eq!(read[3].1.as_ref().unwrap().seq, 10);
    assert_eq!(
        call_path(Path::new("d"), 7, Tool::Look),
        Path::new("d").join("calls").join("0007-look.json")
    );
}

#[test]
fn a_scene_sent_as_a_string_is_kept_as_is() {
    assert_eq!(scene_json(&json!("{\"level\":\"x\"}")), "{\"level\":\"x\"}");
    assert_eq!(scene_json(&json!({ "level": "x" })), "{\"level\":\"x\"}");
}

#[test]
fn a_submission_round_trips() {
    let outcome: Outcome = serde_json::from_str(
        r#"{"goal_met":true,"end":"goal_met","time_s":1.0,"steps":120,"parts_used":1,"chain_length":1,"event_hash":"00000000000000ab","state_hash":"00000000000000cd","rules_version":1}"#,
    )
    .unwrap();
    let submission = Submission {
        level: "bell-drop".to_owned(),
        arm: Arm::Blueprint,
        name: "One plank".to_owned(),
        prediction: "It rings.".to_owned(),
        scene: json!({ "level": "bell-drop", "parts": [] }),
        outcome,
    };
    let text = serde_json::to_string(&submission).unwrap();
    assert!(
        text.contains("\"event_hash\":\"00000000000000ab\""),
        "{text}"
    );
    let back: Submission = serde_json::from_str(&text).unwrap();
    assert_eq!(back, submission);
}

#[test]
fn transcript_records_round_trip_through_jsonl_and_junk_lines_are_skipped() {
    use super::transcript::{Content, Record, from_jsonl, to_jsonl};
    let records = vec![
        Record::Text {
            text: "Looking.".to_owned(),
        },
        Record::ToolUse {
            name: "check".to_owned(),
            input: json!({ "scene": { "level": "bell-drop", "parts": [] } }),
        },
        Record::ToolResult {
            content: vec![
                Content::Text {
                    text: "ok: 0 parts placed".to_owned(),
                },
                Content::Image { omitted: true },
            ],
            is_error: false,
        },
        Record::ToolResult {
            content: vec![Content::Text {
                text: "refused".to_owned(),
            }],
            is_error: true,
        },
        Record::Result {
            subtype: "success".to_owned(),
            num_turns: Some(3),
            duration_ms: Some(1200),
            total_cost_usd: None,
        },
    ];
    let text = to_jsonl(&records);
    assert_eq!(text.lines().count(), 5);
    assert!(!text.lines().nth(2).unwrap().contains("is_error"), "{text}");
    assert!(
        text.lines().nth(3).unwrap().contains("\"is_error\":true"),
        "{text}"
    );
    assert_eq!(from_jsonl(&text), records);
    let noisy = format!("not json\n{text}\n{{\"type\":\"picture\"}}\n\n");
    assert_eq!(from_jsonl(&noisy), records);
    assert!(from_jsonl("").is_empty());
}
