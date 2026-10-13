//! `run_level`: a full run of the bell-drop fixture with its reference solution. Budget: under 200 ms native.
#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::run;
use clattertrap_core::validate::validate;
use criterion::{Criterion, criterion_group, criterion_main};

const LEVEL: &str = include_str!("../tests/fixtures/bell-drop/level.json");
const REFERENCE: &str = include_str!("../tests/fixtures/bell-drop/reference.json");

fn run_level(c: &mut Criterion) {
    let level: Level = serde_json::from_str(LEVEL).unwrap();
    let scene: Scene = serde_json::from_str(REFERENCE).unwrap();
    let valid = validate(&level, &scene, &Rules::V1).unwrap();
    c.bench_function("run_level bell-drop reference", |b| {
        b.iter(|| run(&level, &valid, &Rules::V1));
    });
}

criterion_group!(benches, run_level);
criterion_main!(benches);
