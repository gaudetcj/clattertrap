//! The runner's pure parts: run directories, summaries, the command line, and the model allowlist.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::claude::{command_args, is_claude_model};
use super::next_run_dir;
use crate::mcp::arm::Arm;
use crate::records::{Status, Summary};
use std::path::Path;

#[test]
fn run_directories_count_up_per_model_level_and_arm() {
    let dir = tempfile::tempdir().unwrap();
    let first = next_run_dir(dir.path(), "claude-haiku-5-5", "bell-drop", Arm::Blueprint).unwrap();
    assert_eq!(
        first,
        dir.path()
            .join("claude-haiku-5-5")
            .join("bell-drop")
            .join("blueprint-01")
    );
    std::fs::create_dir_all(&first).unwrap();
    let second = next_run_dir(dir.path(), "claude-haiku-5-5", "bell-drop", Arm::Blueprint).unwrap();
    assert!(second.ends_with(Path::new("bell-drop").join("blueprint-02")));
    let other_arm =
        next_run_dir(dir.path(), "claude-haiku-5-5", "bell-drop", Arm::Workshop).unwrap();
    assert!(other_arm.ends_with("workshop-01"));
}

#[test]
fn summaries_round_trip_with_snake_case_statuses() {
    let summary = Summary {
        model: "claude-opus-5-5".to_owned(),
        level: "bell-drop".to_owned(),
        arm: Arm::Workshop,
        status: Status::NoSubmission,
        started_unix_s: 1_792_267_200,
        duration_s: 94.5,
        cost_usd: Some(1.25),
        turns: Some(12),
        tool_calls: 7,
        foreign_tools: vec![],
        result_subtype: Some("success".to_owned()),
        exit_code: Some(0),
        error: None,
    };
    let json = serde_json::to_string_pretty(&summary).unwrap();
    assert!(json.contains("\"status\": \"no_submission\""), "{json}");
    assert!(json.contains("\"arm\": \"workshop\""), "{json}");
    let back: Summary = serde_json::from_str(&json).unwrap();
    assert_eq!(back, summary);
}

#[test]
fn only_claude_models_pass_the_allowlist() {
    for ok in [
        "haiku",
        "sonnet",
        "opus",
        "fable",
        "claude-haiku-5-5",
        "claude-haiku-4-5",
        "claude-opus-5-5",
    ] {
        assert!(is_claude_model(ok), "{ok}");
    }
    for bad in ["gpt-5", "gemini", "", "claude", "llama3"] {
        assert!(!is_claude_model(bad), "{bad}");
    }
}

#[test]
fn the_command_line_restricts_the_session_to_the_mcp_tools() {
    let args = command_args(
        "claude-haiku-5-5",
        Path::new("C:/runs/bell-drop/workshop-01/mcp.json"),
        6.0,
    );
    let text = args.join(" ");
    assert!(text.contains("--tools  "), "{text}");
    assert!(
        text.contains("--allowedTools mcp__clattertrap__*"),
        "{text}"
    );
    assert!(text.contains("--strict-mcp-config"), "{text}");
    assert!(text.contains("--output-format stream-json"), "{text}");
    assert!(text.contains("--max-budget-usd 6"), "{text}");
    assert!(!text.contains("bypassPermissions"), "{text}");
    insta::assert_snapshot!(args.join("\n"));
}

mod transcript {
    use super::super::transcript::{Transcript, status};
    use crate::records::Status;

    const SMOKE: &str = include_str!("../../tests/fixtures/transcripts/haiku-smoke.jsonl");

    fn parsed(text: &str) -> Transcript {
        Transcript::parse(text)
    }

    #[test]
    fn the_smoke_transcript_parses_into_calls_cost_and_result() {
        let t = parsed(SMOKE);
        assert!(t.mcp_connected);
        assert_eq!(t.model.as_deref(), Some("claude-haiku-5-5"));
        assert_eq!(
            t.tool_calls,
            ["mcp__clattertrap__rules", "mcp__clattertrap__level"]
        );
        assert!(t.foreign_tools().is_empty());
        let result = t.result.as_ref().unwrap();
        assert_eq!(result.subtype, "success");
        assert!(!result.is_error);
        assert_eq!(result.num_turns, Some(3));
        assert!(result.total_cost_usd.unwrap() > 0.0);
        assert_eq!(result.text.as_deref(), Some("done"));
    }

    #[test]
    fn a_session_that_ends_without_a_submission_is_no_submission() {
        let t = parsed(SMOKE);
        assert_eq!(status(&t, false, false), (Status::NoSubmission, None));
        assert_eq!(status(&t, true, false), (Status::Submitted, None));
    }

    #[test]
    fn a_submission_before_the_wall_clock_counts_and_after_it_times_out() {
        let t = parsed("");
        assert_eq!(status(&t, true, true), (Status::Submitted, None));
        let (s, _) = status(&t, false, true);
        assert_eq!(s, Status::TimedOut);
    }

    #[test]
    fn the_budget_cap_is_over_budget_and_other_error_subtypes_abort() {
        let budget = r#"{"type":"result","subtype":"error_max_budget_usd","is_error":true,"num_turns":9,"total_cost_usd":6.01,"result":"Budget exceeded"}"#;
        let (s, _) = status(&parsed(budget), false, false);
        assert_eq!(s, Status::OverBudget);
        let failed = r#"{"type":"result","subtype":"error_during_execution","is_error":true,"num_turns":1,"total_cost_usd":0.1,"result":"API Error: 529 overloaded"}"#;
        assert_eq!(
            status(&parsed(failed), false, false),
            (
                Status::Aborted,
                Some("error_during_execution: API Error: 529 overloaded".to_owned())
            )
        );
    }

    #[test]
    fn a_missing_result_or_a_server_that_never_connected_aborts() {
        let (s, why) = status(&parsed(""), false, false);
        assert_eq!(s, Status::Aborted);
        assert_eq!(why.as_deref(), Some("no result record"));
        let init = r#"{"type":"system","subtype":"init","tools":[],"mcp_servers":[{"name":"clattertrap","status":"failed"}]}
{"type":"result","subtype":"success","is_error":false,"num_turns":1,"total_cost_usd":0.01,"result":"I have no tools."}"#;
        let t = parsed(init);
        assert!(!t.mcp_connected);
        assert_eq!(
            status(&t, false, false),
            (
                Status::Aborted,
                Some("the clattertrap MCP server did not connect".to_owned())
            )
        );
    }

    #[test]
    fn a_tool_outside_the_server_flags_the_run_even_with_a_submission() {
        let text = r#"{"type":"system","subtype":"init","tools":["Bash"],"mcp_servers":[{"name":"clattertrap","status":"connected"}]}
{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"mcp__clattertrap__check","input":{}}]}}
{"type":"assistant","message":{"content":[{"type":"text","text":"Let me look around."},{"type":"tool_use","id":"t2","name":"Bash","input":{"command":"ls"}}]}}
{"type":"result","subtype":"success","is_error":false,"num_turns":4,"total_cost_usd":0.2,"result":"ok"}"#;
        let t = parsed(text);
        assert_eq!(t.tool_calls.len(), 2);
        assert_eq!(t.foreign_tools(), ["Bash"]);
        assert_eq!(
            status(&t, true, false),
            (Status::Flagged, Some("called Bash".to_owned()))
        );
    }

    #[test]
    fn junk_lines_are_skipped() {
        let text = "not json\n{\"type\":\"rate_limit_event\"}\n\n{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"x\"}\n";
        let t = parsed(text);
        assert!(t.result.is_some());
        assert!(t.tool_calls.is_empty());
    }
}

#[test]
fn the_prompt_names_the_level_and_the_run_budget() {
    let text = super::fill_prompt("bell-drop", 8);
    assert!(text.contains("Your level is bell-drop."), "{text}");
    assert!(text.contains("This arm allows 8 runs."), "{text}");
    assert!(!text.contains('{'), "{text}");
}

#[test]
fn the_per_run_config_serves_the_level_into_the_run_directory() {
    let config = super::per_run_config(
        Path::new("C:/bin/clattertrap.exe"),
        "bell-drop",
        Arm::Blueprint,
        Path::new("C:/runs/bell-drop/blueprint-01"),
    );
    let server = &config["mcpServers"]["clattertrap"];
    assert_eq!(server["command"], "C:/bin/clattertrap.exe");
    assert_eq!(
        server["args"],
        serde_json::json!([
            "mcp",
            "--level",
            "bell-drop",
            "--arm",
            "blueprint",
            "--out",
            "C:/runs/bell-drop/blueprint-01"
        ])
    );
}

#[test]
fn the_output_directory_is_made_absolute_before_any_session_starts() {
    let out = super::absolute_out(Path::new("pilot/runs")).unwrap();
    assert!(out.is_absolute(), "{}", out.display());
    assert!(
        out.ends_with(Path::new("pilot").join("runs")),
        "{}",
        out.display()
    );
}
