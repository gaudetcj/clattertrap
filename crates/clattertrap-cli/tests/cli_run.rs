//! The commands that take a scene file: `check`, `look`, and `run`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{clattertrap, repo_root, stdout};
use std::path::PathBuf;

fn scene_path(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scenes")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn reference(id: &str) -> String {
    repo_root()
        .join("levels")
        .join(id)
        .join("reference.json")
        .to_string_lossy()
        .into_owned()
}

/// The golden line of a level's committed reference snapshot, split into words.
fn golden(id: &str) -> Vec<String> {
    let path = repo_root()
        .join("crates")
        .join("clattertrap-core")
        .join("tests")
        .join("snapshots")
        .join(format!("fixtures__{id}-reference.snap"));
    let text = std::fs::read_to_string(path).unwrap();
    let line = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap();
    line.split_whitespace().map(str::to_owned).collect()
}

#[test]
fn check_accepts_a_reference_solution() {
    let out = stdout(&["check", &reference("bell-drop")]);
    assert_eq!(out, "ok: 2 parts placed\n");
}

#[test]
fn check_lists_problems_and_exits_1() {
    for (file, expect) in [
        ("broken.json", "scene: "),
        ("overlap.json", "parts[0] (domino low): overlaps floor"),
        (
            "outside.json",
            "parts[1] (beam far): id \"far\" is already used",
        ),
    ] {
        let out = clattertrap(&["check", &scene_path(file)]);
        assert_eq!(out.status.code(), Some(1), "{file}");
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains(expect), "{file}: {text}");
        insta::assert_snapshot!(file, text);
    }
}

#[test]
fn check_on_an_unknown_level_exits_2() {
    let out = clattertrap(&["check", &scene_path("unknown-level.json")]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("nope"));
}

#[test]
fn run_prints_the_outcome_with_the_golden_hashes() {
    let out = stdout(&["run", &reference("bell-drop")]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let g = golden("bell-drop");
    assert_eq!(v["goal_met"], true);
    assert_eq!(v["event_hash"], g[0]);
    assert_eq!(v["state_hash"], g[1]);
    assert_eq!(v["steps"].to_string(), g[3]);
}

#[test]
fn run_trace_lists_every_event_before_the_outcome() {
    let out = stdout(&["run", "--trace", &reference("bell-drop")]);
    let last_event = out
        .lines()
        .take_while(|l| !l.starts_with('{'))
        .last()
        .unwrap();
    assert!(last_event.ends_with("goal_met"), "{last_event}");
    assert!(out.contains(" contact ball0 b1"), "{out}");
    insta::assert_snapshot!(out);
}

#[test]
fn run_rejects_a_bad_scene_like_check_does() {
    let out = clattertrap(&["run", &scene_path("overlap.json")]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("overlaps floor")
    );
}

#[test]
fn run_png_writes_the_final_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("end.png");
    stdout(&[
        "run",
        &reference("bell-drop"),
        "--png",
        path.to_str().unwrap(),
    ]);
    assert_eq!(&std::fs::read(&path).unwrap()[..4], b"\x89PNG");
}

#[test]
fn look_writes_a_png_next_to_the_scene_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let scene = dir.path().join("mine.json");
    std::fs::copy(reference("bell-drop"), &scene).unwrap();
    let out = stdout(&["look", scene.to_str().unwrap()]);
    let png = dir.path().join("mine.png");
    assert!(out.contains("mine.png"), "{out}");
    assert_eq!(&std::fs::read(&png).unwrap()[..4], b"\x89PNG");
    let elsewhere = dir.path().join("elsewhere.png");
    stdout(&[
        "look",
        scene.to_str().unwrap(),
        "--out",
        elsewhere.to_str().unwrap(),
    ]);
    assert_eq!(&std::fs::read(&elsewhere).unwrap()[..4], b"\x89PNG");
}
