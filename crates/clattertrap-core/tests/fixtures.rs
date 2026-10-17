//! The levels in `levels/`: each reference solution meets its goal, each empty scene does not, and both hash
//! the same on every platform (invariant 1). The committed snapshots are the golden hashes.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::{Run, run};
use clattertrap_core::validate::validate;

/// Fixture id, level JSON, reference JSON, and the committed golden lines for the reference and the empty scene.
struct Fixture {
    id: &'static str,
    level: &'static str,
    reference: &'static str,
    reference_snap: &'static str,
    empty_snap: &'static str,
}

macro_rules! fixture {
    ($id:literal) => {
        Fixture {
            id: $id,
            level: include_str!(concat!("../../../levels/", $id, "/level.json")),
            reference: include_str!(concat!("../../../levels/", $id, "/reference.json")),
            reference_snap: include_str!(concat!("snapshots/fixtures__", $id, "-reference.snap")),
            empty_snap: include_str!(concat!("snapshots/fixtures__", $id, "-empty.snap")),
        }
    };
}

const FIXTURES: [Fixture; 4] = [
    fixture!("bell-drop"),
    fixture!("balloon-pin"),
    fixture!("basket-fill"),
    fixture!("button-drop"),
];

fn load(f: &Fixture) -> (Level, Scene) {
    (
        serde_json::from_str(f.level).unwrap(),
        serde_json::from_str(f.reference).unwrap(),
    )
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

/// The golden line is the last non-empty line of an insta snapshot file.
fn golden_line(snap: &str) -> &str {
    snap.lines().rev().find(|l| !l.trim().is_empty()).unwrap()
}

#[test]
fn every_reference_solution_meets_its_goal() {
    for f in &FIXTURES {
        let (level, reference) = load(f);
        let r = play(&level, &reference);
        eprintln!("{}: {:?}", f.id, r.outcome);
        assert!(
            r.outcome.goal_met,
            "{} reference missed: {:?}",
            f.id, r.outcome
        );
    }
}

#[test]
fn every_empty_scene_misses() {
    for f in &FIXTURES {
        let (level, _) = load(f);
        let r = play(&level, &empty(f.id));
        assert!(
            !r.outcome.goal_met,
            "{} empty scene met the goal: {:?}",
            f.id, r.outcome
        );
    }
}

#[test]
fn running_a_fixture_twice_is_identical() {
    for f in &FIXTURES {
        let (level, reference) = load(f);
        assert_eq!(
            play(&level, &reference),
            play(&level, &reference),
            "{}",
            f.id
        );
    }
}

#[test]
#[cfg(not(target_family = "wasm"))]
fn golden_hashes_hold() {
    for f in &FIXTURES {
        let (level, reference) = load(f);
        insta::assert_snapshot!(
            format!("{}-reference", f.id),
            golden(&play(&level, &reference))
        );
        insta::assert_snapshot!(
            format!("{}-empty", f.id),
            golden(&play(&level, &empty(f.id)))
        );
    }
}

/// The same check as `golden_hashes_hold`, as a plain comparison so it also runs on wasm32.
#[test]
fn golden_hashes_match_the_committed_snapshots() {
    for f in &FIXTURES {
        let (level, reference) = load(f);
        assert_eq!(
            golden(&play(&level, &reference)),
            golden_line(f.reference_snap),
            "{} reference",
            f.id
        );
        assert_eq!(
            golden(&play(&level, &empty(f.id))),
            golden_line(f.empty_snap),
            "{} empty",
            f.id
        );
    }
}
