//! The wrapper replays a level to the committed golden hash on wasm32 and refuses bad input with
//! the validation message.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use clattertrap_wasm::Replay;
use wasm_bindgen_test::wasm_bindgen_test;

const LEVEL: &str = include_str!("../../../levels/bell-drop/level.json");
const REFERENCE: &str = include_str!("../../../levels/bell-drop/reference.json");
const SNAP: &str =
    include_str!("../../clattertrap-core/tests/snapshots/fixtures__bell-drop-reference.snap");

/// The golden line of an insta snapshot: `event_hash state_hash steps N End chain C`.
fn golden() -> Vec<&'static str> {
    SNAP.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap()
        .split_whitespace()
        .collect()
}

#[wasm_bindgen_test]
fn the_bell_drop_reference_replays_to_the_golden_hash() {
    let mut replay = Replay::new(LEVEL, REFERENCE).unwrap();
    let setup: serde_json::Value = serde_json::from_str(&replay.setup().unwrap()).unwrap();
    assert_eq!(setup["width_cm"], 400.0);
    assert_eq!(setup["goal"]["kind"], "ring");
    assert_eq!(setup["names"][1], "bell");
    let slots = setup["slots"].as_array().unwrap().len();
    assert_eq!(slots, 6, "floor, ball, bell, anchor, two beams");
    assert_eq!(setup["slots"][1]["visual"]["shape"], "circle");
    assert_eq!(replay.frame().len(), slots * 4);

    let mut events = 0;
    let mut goal_met_steps = Vec::new();
    while replay.outcome().unwrap().is_none() {
        let step_events = replay.step();
        assert_eq!(step_events.len() % 3, 0);
        events += step_events.len() / 3;
        if step_events.chunks(3).any(|e| e[0] == 2) {
            goal_met_steps.push(replay.step_index());
        }
    }
    let outcome: serde_json::Value =
        serde_json::from_str(&replay.outcome().unwrap().unwrap()).unwrap();
    let g = golden();
    assert_eq!(outcome["event_hash"], g[0]);
    assert_eq!(outcome["state_hash"], g[1]);
    assert_eq!(outcome["steps"].as_u64().unwrap().to_string(), g[3]);
    assert_eq!(outcome["end"], "goal_met");
    assert_eq!(goal_met_steps, [replay.step_index()]);
    assert!(
        events >= 3,
        "at least two contacts and the goal, got {events}"
    );
    assert_eq!(replay.velocity(1).len(), 2, "the ball still exists");
}

#[wasm_bindgen_test]
fn a_scene_outside_the_zone_is_refused_with_its_path() {
    let scene = r#"{ "level": "bell-drop", "parts": [
      { "id": "b1", "kind": "beam", "x_cm": 10, "y_cm": 150, "length_cm": 100, "angle_deg": 0 } ] }"#;
    let Err(err) = Replay::new(LEVEL, scene) else {
        panic!("a beam outside the zone was accepted")
    };
    assert!(err.contains("parts[0]"), "{err}");
    assert!(err.contains("zone"), "{err}");
}

#[wasm_bindgen_test]
fn broken_json_is_refused() {
    assert!(Replay::new("{", REFERENCE).is_err());
    assert!(Replay::new(LEVEL, "{").is_err());
}

#[wasm_bindgen_test]
fn a_preview_draws_an_overlapping_scene_and_lists_the_problem() {
    let scene = r#"{ "level": "bell-drop", "parts": [
      { "id": "d1", "kind": "domino", "x_cm": 200, "y_cm": 0 } ] }"#;
    assert!(
        Replay::new(LEVEL, scene).is_err(),
        "a domino in the floor was accepted"
    );
    let replay = Replay::preview(LEVEL, scene).unwrap();
    let problems = replay.problems();
    assert!(problems.contains("overlaps floor"), "{problems}");
    let setup: serde_json::Value = serde_json::from_str(&replay.setup().unwrap()).unwrap();
    let names = setup["names"].as_array().unwrap();
    assert!(
        names.iter().any(|n| n == "d1"),
        "the domino is placed all the same: {names:?}"
    );
    assert!(Replay::new(LEVEL, REFERENCE).unwrap().problems().is_empty());
}

#[wasm_bindgen_test]
fn a_preview_refuses_a_part_it_cannot_build() {
    let scene = r#"{ "level": "bell-drop", "parts": [
      { "id": "b1", "kind": "beam", "x_cm": 200, "y_cm": 150, "length_cm": 900, "angle_deg": 0 } ] }"#;
    let Err(err) = Replay::preview(LEVEL, scene) else {
        panic!("a beam of 900 cm was built")
    };
    assert!(err.contains("length_cm 900"), "{err}");
}

#[wasm_bindgen_test]
fn the_ranges_come_from_the_rules_in_metres() {
    let ranges: serde_json::Value = serde_json::from_str(&clattertrap_wasm::ranges()).unwrap();
    assert_eq!(ranges["beam_length"]["min_m"], 0.2);
    assert_eq!(ranges["beam_length"]["max_m"], 1.5);
    assert_eq!(ranges["funnel_throat_min_m"], 0.14);
    assert_eq!(ranges["rope_length"]["max_m"], 2.0);
}
