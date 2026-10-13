//! `run_level`: a full run of the bell-drop fixture with its reference solution. Budget: under 200 ms native.
#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::run;
use clattertrap_core::validate::validate;
use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;

fn load(id: &str) -> (Level, Scene) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(id);
    let level =
        serde_json::from_str(&std::fs::read_to_string(dir.join("level.json")).unwrap()).unwrap();
    let scene = serde_json::from_str(&std::fs::read_to_string(dir.join("reference.json")).unwrap())
        .unwrap();
    (level, scene)
}

fn run_level(c: &mut Criterion) {
    let (level, scene) = load("bell-drop");
    let valid = validate(&level, &scene, &Rules::V1).unwrap();
    c.bench_function("run_level bell-drop reference", |b| {
        b.iter(|| run(&level, &valid, &Rules::V1));
    });
}

criterion_group!(benches, run_level);
criterion_main!(benches);
