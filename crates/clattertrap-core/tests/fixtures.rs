//! Fixture levels: each reference solution meets its goal, each empty scene does not, and both hash the same
//! on every platform (invariant 1). The snapshots are the golden hashes.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::{Run, run};
use clattertrap_core::validate::validate;
use std::path::{Path, PathBuf};

const FIXTURES: [&str; 4] = ["bell-drop", "balloon-pin", "basket-fill", "button-drop"];

fn dir(id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(id)
}

fn load(id: &str) -> (Level, Scene) {
    let level = serde_json::from_str(&std::fs::read_to_string(dir(id).join("level.json")).unwrap())
        .unwrap();
    let reference =
        serde_json::from_str(&std::fs::read_to_string(dir(id).join("reference.json")).unwrap())
            .unwrap();
    (level, reference)
}

fn empty(id: &str) -> Scene {
    Scene {
        level: id.to_owned(),
        parts: vec![],
    }
}

fn play(level: &Level, scene: &Scene) -> Run {
    let valid =
        validate(level, scene, &Rules::V1).unwrap_or_else(|p| panic!("{}: {p:#?}", level.id));
    run(level, &valid, &Rules::V1)
}

fn golden(r: &Run) -> String {
    let o = &r.outcome;
    format!(
        "{:016x} {:016x} steps {} {:?} chain {}",
        o.event_hash, o.state_hash, o.steps, o.end, o.chain_length
    )
}

#[test]
fn every_reference_solution_meets_its_goal() {
    for id in FIXTURES {
        let (level, reference) = load(id);
        let r = play(&level, &reference);
        eprintln!("{id}: {:?}", r.outcome);
        assert!(r.outcome.goal_met, "{id} reference missed: {:?}", r.outcome);
    }
}

#[test]
fn every_empty_scene_misses() {
    for id in FIXTURES {
        let (level, _) = load(id);
        let r = play(&level, &empty(id));
        assert!(
            !r.outcome.goal_met,
            "{id} empty scene met the goal: {:?}",
            r.outcome
        );
    }
}

#[test]
fn running_a_fixture_twice_is_identical() {
    for id in FIXTURES {
        let (level, reference) = load(id);
        assert_eq!(play(&level, &reference), play(&level, &reference), "{id}");
    }
}

#[test]
fn golden_hashes_hold() {
    for id in FIXTURES {
        let (level, reference) = load(id);
        insta::assert_snapshot!(format!("{id}-reference"), golden(&play(&level, &reference)));
        insta::assert_snapshot!(format!("{id}-empty"), golden(&play(&level, &empty(id))));
    }
}
