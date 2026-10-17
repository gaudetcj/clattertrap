//! `clattertrap mcp` end to end: a client over the child process calling every tool. The answers
//! are what contestants read, so they are snapshotted.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::mcp::Client;
use serde_json::{Value, json};
use std::fmt::Write;

fn reference(id: &str) -> Value {
    let path = common::levels_dir().join(id).join("reference.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[tokio::test]
async fn the_server_lists_the_seven_contestant_tools() {
    let client = Client::start(&[]).await;
    let tools = client.tools().await;
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        ["check", "level", "levels", "look", "rules", "run", "submit"]
    );
    let mut listing = String::new();
    for tool in &tools {
        let schema = serde_json::to_string(&Value::Object((*tool.input_schema).clone())).unwrap();
        writeln!(
            listing,
            "{}: {}\n  {schema}",
            tool.name,
            tool.description.as_deref().unwrap_or("")
        )
        .unwrap();
    }
    insta::assert_snapshot!(listing);
    client.close().await;
}

#[tokio::test]
async fn a_workshop_session_runs_and_submits() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().to_str().unwrap();
    let client = Client::start(&["--level", "bell-drop", "--arm", "workshop", "--out", out]).await;

    let rules = client.text("rules", json!({})).await;
    assert!(rules.starts_with("Clattertrap rules v1"), "{rules}");
    let schema = client.text("rules", json!({ "schema": true })).await;
    assert!(schema.contains("\"title\": \"Scene\""), "{schema}");

    let level = client.text("level", json!({ "id": "bell-drop" })).await;
    assert!(
        level.starts_with("bell-drop: Ring the bell (tier 1)"),
        "{level}"
    );
    assert!(level.contains("<png "), "{level}");
    let other = client.text("level", json!({ "id": "grand-tour" })).await;
    assert_eq!(
        other,
        "error:\nyour level is bell-drop; call level with id \"bell-drop\"\n"
    );

    let check = client
        .text("check", json!({ "scene": reference("bell-drop") }))
        .await;
    assert_eq!(check, "ok: 2 parts placed\n");

    let look = client
        .text("look", json!({ "scene": reference("bell-drop") }))
        .await;
    assert!(look.contains("<png "), "{look}");

    let run = client
        .text("run", json!({ "scene": reference("bell-drop") }))
        .await;
    assert!(run.contains("runs left: 7"), "{run}");
    insta::assert_snapshot!(run.split("<png").next().unwrap());

    let submit = client
        .text(
            "submit",
            json!({
                "scene": reference("bell-drop"),
                "name": "Two planks",
                "prediction": "The ball rolls down both beams and rings the bell."
            }),
        )
        .await;
    insta::assert_snapshot!(submit);
    let again = client
        .text("check", json!({ "scene": reference("bell-drop") }))
        .await;
    assert_eq!(again, "error:\nalready submitted; the session is over\n");
    client.close().await;

    assert!(dir.path().join("submission.json").is_file());
    let calls: Vec<String> = std::fs::read_dir(dir.path().join("calls"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(calls.len(), 4, "{calls:?}");
}

#[tokio::test]
async fn a_blueprint_session_refuses_to_run() {
    let client = Client::start(&["--level", "bell-drop", "--arm", "blueprint"]).await;
    let run = client
        .text("run", json!({ "scene": reference("bell-drop") }))
        .await;
    assert_eq!(
        run,
        "error:\nthis arm allows 0 runs; use check and look, then submit\n"
    );
    client.close().await;
}

#[tokio::test]
async fn a_bad_scene_lists_problems_without_an_error() {
    let client = Client::start(&[]).await;
    let text = client
        .text(
            "check",
            json!({ "scene": { "level": "bell-drop", "parts": [
                { "id": "low", "kind": "domino", "x_cm": 100, "y_cm": 5 }
            ] } }),
        )
        .await;
    assert_eq!(
        text,
        "parts[0] (domino low): overlaps floor by 5.0 cm at the start; move it up to y_cm = 10.0\n"
    );
    client.close().await;
}
