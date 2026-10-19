//! `clattertrap arena --endpoint` end to end against a recorded chat endpoint: the real binary, the
//! real MCP server, and scripted replies.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::fake_endpoint::{Fake, serve};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn reference() -> Value {
    let path = common::levels_dir()
        .join("bell-drop")
        .join("reference.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// A reply that calls the named tools with JSON-text arguments and reports `cost`.
fn calls(text: &str, tools: &[(&str, Value)], cost: f64) -> Value {
    let tool_calls: Vec<Value> = tools
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({ "id": format!("call_{i}"), "type": "function",
                "function": { "name": name, "arguments": args.to_string() } })
        })
        .collect();
    json!({ "choices": [{ "finish_reason": "tool_calls",
        "message": { "role": "assistant", "content": text, "tool_calls": tool_calls } }],
        "usage": { "cost": cost } })
}

fn stop(text: &str, cost: f64) -> Value {
    json!({ "choices": [{ "finish_reason": "stop", "message": { "role": "assistant", "content": text } }],
        "usage": { "cost": cost } })
}

fn arena(fake: &Fake, out: &Path, key: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clattertrap"));
    cmd.args([
        "arena",
        "--model",
        "Fake/Model-1",
        "--arm",
        "blueprint",
        "--level",
        "bell-drop",
        "--out",
        out.to_str().unwrap(),
        "--endpoint",
        &fake.url,
    ])
    .env_remove("OPENROUTER_API_KEY")
    .current_dir(common::repo_root());
    if let Some(key) = key {
        cmd.env("OPENROUTER_API_KEY", key);
    }
    cmd.output().unwrap()
}

fn run_dir(out: &Path) -> PathBuf {
    out.join("fake-model-1")
        .join("bell-drop")
        .join("blueprint-01")
}

fn summary(out: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(run_dir(out).join("summary.json")).unwrap())
        .unwrap()
}

fn records(out: &Path) -> Vec<Value> {
    std::fs::read_to_string(run_dir(out).join("transcript.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn kinds(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .map(|r| match r["type"].as_str().unwrap() {
            "tool_use" => format!("tool_use {}", r["name"].as_str().unwrap()),
            "tool_result" if r["is_error"] == true => "tool_result error".to_owned(),
            other => other.to_owned(),
        })
        .collect()
}

/// Runs the four-reply script, rules and level, check, submit, stop, with a key in the environment.
fn scripted() -> (Fake, tempfile::TempDir) {
    let scene = reference();
    let fake = serve(vec![
        (
            200,
            calls(
                "Reading.",
                &[
                    ("rules", json!({})),
                    ("level", json!({ "id": "bell-drop" })),
                ],
                0.01,
            ),
        ),
        (
            200,
            calls("Checking.", &[("check", json!({ "scene": scene }))], 0.02),
        ),
        (
            200,
            calls(
                "",
                &[(
                    "submit",
                    json!({ "scene": scene, "name": "Reference drop",
            "prediction": "The ball rolls down the plank and rings the bell." }),
                )],
                0.03,
            ),
        ),
        (200, stop("Done.", 0.01)),
    ]);
    let dir = tempfile::tempdir().unwrap();
    let out = arena(&fake, dir.path(), Some("test-key"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let line = String::from_utf8_lossy(&out.stdout);
    assert!(
        line.contains("bell-drop blueprint-01 submitted goal_met 0.07 USD"),
        "{line}"
    );
    (fake, dir)
}

#[test]
fn a_scripted_session_checks_submits_and_is_recorded_like_a_claude_run() {
    let (_fake, dir) = scripted();
    let s = summary(dir.path());
    assert_eq!(s["model"], "fake-model-1");
    assert_eq!(s["status"], "submitted");
    assert_eq!(s["turns"], 4);
    assert_eq!(s["tool_calls"], 4);
    assert_eq!(s["foreign_tools"], json!([]));
    assert_eq!(s["result_subtype"], "success");
    assert_eq!(s["exit_code"], Value::Null);
    assert_eq!(s["error"], Value::Null);
    assert!((s["cost_usd"].as_f64().unwrap() - 0.07).abs() < 1e-9);

    let run = run_dir(dir.path());
    for file in [
        "mcp.json",
        "provider.jsonl",
        "stderr.txt",
        "calls/0001-check.json",
        "calls/0002-submit.json",
    ] {
        assert!(run.join(file).is_file(), "{file}");
    }
    let submission: Value =
        serde_json::from_str(&std::fs::read_to_string(run.join("submission.json")).unwrap())
            .unwrap();
    assert_eq!(submission["name"], "Reference drop");
    assert_eq!(submission["outcome"]["goal_met"], true);
    assert_eq!(
        std::fs::read_to_string(run.join("provider.jsonl"))
            .unwrap()
            .lines()
            .count(),
        4
    );

    let records = records(dir.path());
    assert_eq!(
        kinds(&records),
        [
            "text",
            "tool_use rules",
            "tool_result",
            "tool_use level",
            "tool_result",
            "text",
            "tool_use check",
            "tool_result",
            "tool_use submit",
            "tool_result",
            "text",
            "result",
        ]
    );
    // The level answer carries a picture, kept as the marker.
    assert_eq!(
        records[4]["content"][1],
        json!({ "type": "image", "omitted": true })
    );
    assert_eq!(records[6]["input"]["scene"]["level"], "bell-drop");
    assert_eq!(records[11]["subtype"], "success");
    assert_eq!(records[11]["num_turns"], 4);
}

#[test]
fn the_requests_carry_the_key_the_tools_the_history_and_the_pictures() {
    let (fake, _dir) = scripted();
    let seen = fake.seen.lock().unwrap();
    assert_eq!(seen.len(), 4);
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer test-key"));
    let first = &seen[0].body;
    assert_eq!(first["model"], "Fake/Model-1");
    let names: Vec<&str> = first["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["function"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["check", "level", "levels", "look", "rules", "run", "submit"]
    );
    assert!(
        first["tools"][0]["function"]["parameters"]
            .get("$schema")
            .is_none()
    );
    assert_eq!(first["messages"][0]["role"], "system");
    assert!(
        first["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains("Your level is bell-drop."),
        "{first}"
    );
    // The second request carries the echoed assistant message, both tool answers, and the picture.
    let second = seen[1].body["messages"].as_array().unwrap();
    assert_eq!(second[2]["role"], "assistant");
    assert_eq!(second[2]["tool_calls"][1]["id"], "call_1");
    assert_eq!(second[3]["role"], "tool");
    assert_eq!(second[3]["tool_call_id"], "call_0");
    assert_eq!(second[4]["tool_call_id"], "call_1");
    assert_eq!(second[5]["role"], "user");
    assert!(
        second[5]["content"][1]["image_url"]["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"),
        "{}",
        second[5]
    );
    assert_eq!(second.len(), 6);
}

#[test]
fn a_server_error_is_retried_and_a_session_that_stops_without_submitting_counts() {
    let fake = serve(vec![
        (500, json!({ "error": { "message": "overloaded" } })),
        (200, stop("I give up.", 0.001)),
    ]);
    let dir = tempfile::tempdir().unwrap();
    let out = arena(&fake, dir.path(), None);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = summary(dir.path());
    assert_eq!(s["status"], "no_submission");
    assert_eq!(s["turns"], 1);
    assert_eq!(s["error"], Value::Null);
    assert_eq!(fake.seen.lock().unwrap()[0].authorization, None);
    assert_eq!(fake.seen.lock().unwrap().len(), 2);
}

#[test]
fn the_dollar_cap_ends_the_session_over_budget() {
    let fake = serve(vec![(
        200,
        calls("Spending.", &[("rules", json!({}))], 6.5),
    )]);
    let dir = tempfile::tempdir().unwrap();
    assert!(arena(&fake, dir.path(), None).status.success());
    let s = summary(dir.path());
    assert_eq!(s["status"], "over_budget");
    assert_eq!(s["result_subtype"], "error_max_budget_usd");
    assert_eq!(s["tool_calls"], 0, "no tool runs after the cap");
}

#[test]
fn a_tool_outside_the_server_flags_the_run() {
    let fake = serve(vec![(
        200,
        calls("", &[("Bash", json!({ "command": "ls" }))], 0.01),
    )]);
    let dir = tempfile::tempdir().unwrap();
    let out = arena(&fake, dir.path(), None);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("to run again"));
    let s = summary(dir.path());
    assert_eq!(s["status"], "flagged");
    assert_eq!(s["foreign_tools"], json!(["Bash"]));
    assert_eq!(s["error"], "called Bash");
}

#[test]
fn a_rejected_request_aborts_the_run_with_the_endpoint_message() {
    let fake = serve(vec![(
        400,
        json!({ "error": { "message": "images are not supported" } }),
    )]);
    let dir = tempfile::tempdir().unwrap();
    assert!(arena(&fake, dir.path(), None).status.success());
    let s = summary(dir.path());
    assert_eq!(s["status"], "aborted");
    assert_eq!(s["result_subtype"], "error_api");
    let error = s["error"].as_str().unwrap();
    assert!(error.starts_with("error_api: HTTP 400"), "{error}");
    assert!(error.contains("images are not supported"), "{error}");
}

#[test]
fn a_reply_the_provider_cut_short_aborts_the_run_instead_of_counting_a_miss() {
    let cut = json!({ "choices": [{ "finish_reason": "error", "error": { "message": "upstream timed out" },
        "message": { "role": "assistant", "content": "" } }], "usage": { "cost": 0.01 } });
    let fake = serve(vec![
        (200, calls("Reading.", &[("rules", json!({}))], 0.01)),
        (200, cut),
    ]);
    let dir = tempfile::tempdir().unwrap();
    assert!(arena(&fake, dir.path(), None).status.success());
    let s = summary(dir.path());
    assert_eq!(s["status"], "aborted");
    assert_eq!(s["result_subtype"], "error_api");
    assert_eq!(
        s["error"],
        "error_api: the endpoint cut the reply short: upstream timed out"
    );
}
