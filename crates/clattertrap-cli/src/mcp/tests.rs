//! The session behind every MCP tool, exercised without the protocol.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::arm::Arm;
use super::session::{Reply, Session, ToolError};
use clattertrap_core::level::Level;
use clattertrap_core::validate::parse_level;
use serde_json::{Value, json};
use std::path::Path;

const BELL_LEVEL: &str = include_str!("../../../../levels/bell-drop/level.json");
const BELL_REFERENCE: &str = include_str!("../../../../levels/bell-drop/reference.json");

fn bell() -> Level {
    parse_level(BELL_LEVEL).unwrap()
}

fn reference() -> Value {
    serde_json::from_str(BELL_REFERENCE).unwrap()
}

fn overlapping() -> Value {
    json!({ "level": "bell-drop", "parts": [
        { "id": "low", "kind": "domino", "x_cm": 100, "y_cm": 5 }
    ] })
}

fn workshop(out: Option<&Path>) -> Session {
    Session::new(Some(bell()), Arm::Workshop, out.map(Path::to_path_buf))
}

fn refused(result: Result<Reply, ToolError>) -> String {
    match result {
        Err(ToolError::Refused(text)) => text,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn ok(result: Result<Reply, ToolError>) -> Reply {
    result.expect("the tool should answer")
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG")
}

#[test]
fn rules_gives_the_text_or_the_scene_schema() {
    let text = ok(Session::rules(false)).text;
    assert!(text.starts_with("Clattertrap rules v1"), "{text}");
    let schema: Value = serde_json::from_str(&ok(Session::rules(true)).text).unwrap();
    assert_eq!(schema["title"], "Scene");
}

#[test]
fn levels_lists_every_level_by_tier() {
    let text = ok(Session::levels()).text;
    assert!(text.starts_with("bell-drop "), "{text}");
    assert_eq!(text.lines().count(), 15);
}

#[test]
fn level_gives_the_text_and_a_png() {
    let reply = ok(workshop(None).level("bell-drop"));
    assert!(
        reply.text.starts_with("bell-drop: Ring the bell (tier 1)"),
        "{}",
        reply.text
    );
    assert!(is_png(&reply.png.unwrap()));
}

#[test]
fn level_refuses_another_id_in_a_pilot_session() {
    let text = refused(workshop(None).level("grand-tour"));
    assert_eq!(
        text,
        "your level is bell-drop; call level with id \"bell-drop\""
    );
    let free = Session::new(None, Arm::Workshop, None);
    assert!(ok(free.level("grand-tour")).text.starts_with("grand-tour:"));
}

#[test]
fn check_accepts_the_reference_and_lists_problems() {
    let mut session = workshop(None);
    assert_eq!(ok(session.check(&reference())).text, "ok: 2 parts placed");
    let problems = ok(session.check(&overlapping())).text;
    assert!(problems.contains("overlaps floor"), "{problems}");
    insta::assert_snapshot!(problems);
}

#[test]
fn check_takes_the_scene_as_a_json_string_too() {
    let mut session = workshop(None);
    let text = ok(session.check(&Value::String(BELL_REFERENCE.to_owned()))).text;
    assert_eq!(text, "ok: 2 parts placed");
}

#[test]
fn look_gives_a_png_for_a_valid_scene_and_problems_otherwise() {
    let mut session = workshop(None);
    let reply = ok(session.look(&reference()));
    assert!(is_png(&reply.png.unwrap()));
    let reply = ok(session.look(&overlapping()));
    assert!(reply.png.is_none());
    assert!(reply.text.contains("overlaps floor"), "{}", reply.text);
}

#[test]
fn run_gives_the_trace_the_outcome_the_runs_left_and_a_png() {
    let mut session = workshop(None);
    let reply = ok(session.run(&reference()));
    assert!(reply.text.contains("  1.925 goal_met"), "{}", reply.text);
    assert!(reply.text.contains("\"goal_met\": true"), "{}", reply.text);
    assert!(reply.text.ends_with("runs left: 7\n"), "{}", reply.text);
    assert!(is_png(&reply.png.unwrap()));
    insta::assert_snapshot!(reply.text);
}

#[test]
fn run_with_problems_costs_no_budget() {
    let mut session = workshop(None);
    let reply = ok(session.run(&overlapping()));
    assert!(reply.text.contains("overlaps floor"), "{}", reply.text);
    assert!(reply.png.is_none());
    let reply = ok(session.run(&reference()));
    assert!(reply.text.ends_with("runs left: 7\n"), "{}", reply.text);
}

#[test]
fn run_refuses_past_the_budget() {
    let mut blind = Session::new(Some(bell()), Arm::Blueprint, None);
    assert_eq!(
        refused(blind.run(&reference())),
        "this arm allows 0 runs; use check and look, then submit"
    );
    let mut session = workshop(None);
    for _ in 0..8 {
        ok(session.run(&reference()));
    }
    assert_eq!(
        refused(session.run(&reference())),
        "this arm allows 8 runs and you have used 8; submit"
    );
}

#[test]
fn submit_works_once_and_ends_the_session() {
    let mut session = workshop(None);
    let reply = ok(session.submit(
        &reference(),
        "Two planks",
        "The ball rolls down both beams and rings the bell.",
    ));
    assert!(reply.text.contains("\"goal_met\": true"), "{}", reply.text);
    assert!(reply.png.is_none());
    insta::assert_snapshot!(reply.text);
    let again = "already submitted; the session is over";
    assert_eq!(refused(session.submit(&reference(), "x", "y")), again);
    assert_eq!(refused(session.run(&reference())), again);
    assert_eq!(refused(session.check(&reference())), again);
    assert_eq!(refused(session.look(&reference())), again);
}

#[test]
fn submit_refuses_a_long_name_or_prediction_or_an_empty_one() {
    let mut session = workshop(None);
    let long_name = "n".repeat(61);
    assert_eq!(
        refused(session.submit(&reference(), &long_name, "p")),
        "name: 61 characters is over the 60 allowed; shorten it"
    );
    let long_prediction = "p".repeat(281);
    assert_eq!(
        refused(session.submit(&reference(), "n", &long_prediction)),
        "prediction: 281 characters is over the 280 allowed; shorten it"
    );
    assert_eq!(
        refused(session.submit(&reference(), " ", "p")),
        "name: is empty; name the machine"
    );
    assert_eq!(
        refused(session.submit(&reference(), "n", "")),
        "prediction: is empty; predict in one sentence what the machine will do"
    );
    assert!(!session.submitted());
}

#[test]
fn submit_with_problems_leaves_the_session_open() {
    let mut session = workshop(None);
    let reply = ok(session.submit(&overlapping(), "n", "p"));
    assert!(reply.text.contains("overlaps floor"), "{}", reply.text);
    assert!(!session.submitted());
}

#[test]
fn every_scene_call_is_recorded_and_submit_writes_the_submission() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = workshop(Some(dir.path()));
    ok(session.check(&overlapping()));
    ok(session.run(&reference()));
    ok(session.look(&reference()));
    ok(session.submit(&reference(), "Two planks", "It rings."));
    let calls = dir.path().join("calls");
    let mut names: Vec<String> = std::fs::read_dir(&calls)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "0001-check.json",
            "0002-run.json",
            "0003-look.json",
            "0004-submit.json"
        ]
    );
    let check: Value =
        serde_json::from_str(&std::fs::read_to_string(calls.join("0001-check.json")).unwrap())
            .unwrap();
    assert_eq!(check["seq"], 1);
    assert_eq!(check["tool"], "check");
    assert_eq!(check["scene"]["level"], "bell-drop");
    assert!(
        check["problems"][0]
            .as_str()
            .unwrap()
            .contains("overlaps floor")
    );
    assert!(check.get("outcome").is_none());
    let run: Value =
        serde_json::from_str(&std::fs::read_to_string(calls.join("0002-run.json")).unwrap())
            .unwrap();
    assert_eq!(run["outcome"]["goal_met"], true);
    assert!(run.get("problems").is_none());
    let submission: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("submission.json")).unwrap())
            .unwrap();
    assert_eq!(submission["level"], "bell-drop");
    assert_eq!(submission["arm"], "workshop");
    assert_eq!(submission["name"], "Two planks");
    assert_eq!(submission["prediction"], "It rings.");
    assert_eq!(submission["scene"]["parts"].as_array().unwrap().len(), 2);
    assert_eq!(submission["outcome"]["goal_met"], true);
    assert_eq!(submission["outcome"]["event_hash"], "41a3e9905cb8f870");
}

#[test]
fn an_unknown_level_in_a_free_session_is_a_problem_not_a_crash() {
    let mut session = Session::new(None, Arm::Workshop, None);
    let text = ok(session.check(&json!({ "level": "nope", "parts": [] }))).text;
    assert!(text.starts_with("level: no level \"nope\";"), "{text}");
}

#[test]
fn the_tool_names_match_what_the_router_lists() {
    let mut listed = super::handler::Handler::listed_tools();
    listed.sort_unstable();
    let mut named: Vec<String> = super::TOOL_NAMES.iter().map(|n| (*n).to_owned()).collect();
    named.sort_unstable();
    assert_eq!(listed, named);
}
