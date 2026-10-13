//! Property tests: every valid scene simulates without NaN, and simulation is pure.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::rules::Rules;
use clattertrap_core::scene::{Material, Placed};
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::validate;
use common::{ball, beam, level, scene};
use proptest::prelude::*;

fn beams() -> impl Strategy<Value = Vec<Placed>> {
    prop::collection::vec(
        (40f32..360.0, 40f32..260.0, 20f32..150.0, -60f32..60.0),
        0..4,
    )
    .prop_map(|v| {
        v.into_iter()
            .enumerate()
            .map(|(i, (x, y, len, angle))| beam(&format!("b{i}"), x, y, len, angle))
            .collect()
    })
}

fn balls() -> impl Strategy<Value = Vec<Placed>> {
    let material = prop_oneof![
        Just(Material::Steel),
        Just(Material::Wood),
        Just(Material::Rubber)
    ];
    prop::collection::vec((40f32..360.0, 150f32..290.0, material), 1..3).prop_map(|v| {
        v.into_iter()
            .enumerate()
            .map(|(i, (x, y, m))| ball(&format!("k{i}"), x, y, m))
            .collect()
    })
}

fn play(
    level: &clattertrap_core::level::Level,
    valid: &clattertrap_core::validate::ValidScene,
) -> Simulator {
    let mut sim = Simulator::new(level, valid, &Rules::V1);
    while sim.finished().is_none() {
        sim.step();
    }
    sim
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn valid_scenes_simulate_without_nan_and_the_same_way_twice(bs in beams(), ks in balls()) {
        let lv = level(ks);
        let sc = scene(bs);
        // Random placements that overlap at the start are rejected by validate and skipped here.
        let Ok(valid) = validate(&lv, &sc, &Rules::V1) else { return Ok(()) };
        let first = play(&lv, &valid);
        for p in first.poses() {
            prop_assert!(p.x_m.is_finite() && p.y_m.is_finite() && p.angle_rad.is_finite(), "{p:?}");
        }
        let second = play(&lv, &valid);
        prop_assert_eq!(first.step_index(), second.step_index());
        prop_assert_eq!(first.state_hash(), second.state_hash());
    }
}
