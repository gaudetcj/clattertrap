//! The runner's pure parts: run directories, summaries, the command line, and the model allowlist.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::claude::{command_args, is_claude_model};
use super::next_run_dir;

mod chat;
mod status;
mod stream;
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

#[test]
fn model_ids_become_directory_safe_slugs() {
    use super::model_slug;
    assert_eq!(model_slug("claude-opus-5-5"), "claude-opus-5-5");
    assert_eq!(model_slug("openai/gpt-5-mini"), "openai-gpt-5-mini");
    assert_eq!(model_slug("qwen/qwen3:free"), "qwen-qwen3-free");
    assert_eq!(model_slug("Meta Llama 3.1"), "meta-llama-3.1");
    assert_eq!(model_slug("--x__y--"), "x-y");
    assert_eq!(model_slug("///"), "");
}
