//! Invariant 1: the same level, scene, and rules give a bit-identical event log and state hash.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use clattertrap_core::sim::run;
use clattertrap_core::validate::validate;
use common::{ball, beam, level, scene};

#[test]
fn the_same_scene_run_twice_gives_identical_events_and_hashes() {
    let lv = level(vec![
        ball("b", 258.0, 131.0, Material::Wood),
        ball("c", 100.0, 150.0, Material::Rubber),
    ]);
    let sc = scene(vec![
        beam("ramp", 200.0, 100.0, 150.0, 20.0),
        beam("shelf", 120.0, 60.0, 100.0, 0.0),
    ]);
    let valid = validate(&lv, &sc, &Rules::V1).unwrap();
    let first = run(&lv, &valid, &Rules::V1);
    let second = run(&lv, &valid, &Rules::V1);
    assert!(
        !first.events.is_empty(),
        "a rolling ball should log contacts"
    );
    assert_eq!(first.events, second.events);
    assert_eq!(first.outcome, second.outcome);
    assert_ne!(first.outcome.state_hash, 0);
}
