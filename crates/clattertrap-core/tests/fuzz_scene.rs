//! Fuzz target `scene`: arbitrary bytes, arbitrary JSON trees, scene-shaped JSON, and mutated reference
//! scenes all go through `check` without a panic, and whatever it accepts simulates to a finite state.
//! nextest runs a few cases; `cargo xtask fuzz scene <secs>` runs the ignored test for a time budget.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::check;
use common::levels::ALL;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use serde_json::{Number, Value};
use std::time::{Duration, Instant};

const KINDS: [&str; 17] = [
    "beam",
    "funnel",
    "ball",
    "domino",
    "crate",
    "seesaw",
    "pendulum",
    "spring_pad",
    "bucket",
    "balloon",
    "pin",
    "rope",
    "bell",
    "button",
    "basket",
    "trampoline",
    "",
];

const FIELDS: [&str; 16] = [
    "x_cm",
    "y_cm",
    "length_cm",
    "angle_deg",
    "material",
    "mouth_cm",
    "throat_cm",
    "height_cm",
    "width_cm",
    "side_cm",
    "max_length_cm",
    "a",
    "b",
    "rope_cm",
    "travel_cm",
    "count",
];

fn level(index: usize) -> Level {
    serde_json::from_str(ALL[index].level).unwrap()
}

/// `check` returns, and an accepted scene simulates to finite poses.
fn holds(level: &Level, json: &str) {
    match check(level, json, &Rules::V1) {
        Ok(valid) => {
            let mut sim = Simulator::new(level, &valid, &Rules::V1);
            while sim.closing().is_none() {
                sim.step();
            }
            for pose in sim.poses() {
                assert!(
                    pose.x_m.is_finite() && pose.y_m.is_finite() && pose.angle_rad.is_finite(),
                    "{json}"
                );
            }
        }
        Err(problems) => assert!(!problems.is_empty(), "rejected with no problem: {json}"),
    }
}

fn number() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<f64>().prop_map(|f| Number::from_f64(f).map_or(Value::Null, Value::Number)),
        any::<i64>().prop_map(Value::from),
        (-500.0f64..500.0).prop_map(|f| Number::from_f64(f).map_or(Value::Null, Value::Number)),
    ]
}

fn json_tree() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        number(),
        "[a-z_0-9]{0,8}".prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            prop::collection::vec(("[a-z_]{1,10}", inner), 0..8)
                .prop_map(|kv| Value::Object(kv.into_iter().collect())),
        ]
    })
}

/// Objects that look like parts: a kind, an id, and some fields with numbers or junk in them.
fn part_shaped() -> impl Strategy<Value = Value> {
    let field = (
        prop::sample::select(FIELDS.to_vec()),
        prop_oneof![
            number(),
            "[a-z]{0,6}".prop_map(Value::String),
            Just(Value::Null)
        ],
    );
    (
        "[a-z][a-z0-9]{0,4}",
        prop::sample::select(KINDS.to_vec()),
        prop::collection::vec(field, 0..7),
    )
        .prop_map(|(id, kind, fields)| {
            let mut obj = serde_json::Map::new();
            obj.insert("id".to_owned(), Value::String(id));
            obj.insert("kind".to_owned(), Value::String(kind.to_owned()));
            for (name, value) in fields {
                obj.insert(name.to_owned(), value);
            }
            Value::Object(obj)
        })
}

fn scene_shaped() -> impl Strategy<Value = (usize, String)> {
    (0..ALL.len(), prop::collection::vec(part_shaped(), 0..6)).prop_map(|(i, parts)| {
        let scene = serde_json::json!({ "level": ALL[i].id, "parts": parts });
        (i, scene.to_string())
    })
}

/// A reference scene with up to six byte-level mutations.
fn mutated_reference() -> impl Strategy<Value = (usize, String)> {
    let ops = prop::collection::vec((any::<usize>(), 0..8u8, any::<u8>()), 0..6);
    (0..ALL.len(), ops).prop_map(|(i, ops)| {
        let mut bytes = ALL[i].reference.as_bytes().to_vec();
        for (pos, op, b) in ops {
            if bytes.is_empty() {
                break;
            }
            let p = pos % bytes.len();
            match op {
                0 => {
                    bytes.remove(p);
                }
                1 => bytes.insert(p, b),
                2 => bytes[p] = b,
                3 => bytes[p] = b'9',
                4 => bytes.truncate(p),
                5 => {
                    let chunk: Vec<u8> = bytes[p..(p + 8).min(bytes.len())].to_vec();
                    bytes.splice(p..p, chunk);
                }
                6 => {
                    bytes.splice(p..p, b"1e30".iter().copied());
                }
                _ => bytes.insert(p, b'-'),
            }
        }
        (i, String::from_utf8_lossy(&bytes).into_owned())
    })
}

proptest! {
    #![proptest_config(Config { cases: 64, ..Config::default() })]

    #[test]
    fn bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        holds(&level(0), &String::from_utf8_lossy(&bytes));
    }

    #[test]
    fn json_trees_never_panic(tree in json_tree()) {
        holds(&level(0), &tree.to_string());
    }

    #[test]
    fn scene_shaped_json_never_panics((i, json) in scene_shaped()) {
        holds(&level(i), &json);
    }

    #[test]
    fn mutated_references_never_panic((i, json) in mutated_reference()) {
        holds(&level(i), &json);
    }
}

/// The long form behind `cargo xtask fuzz scene <secs>`: the same strategies until the budget is spent.
#[test]
#[ignore = "runs for CLATTERTRAP_FUZZ_SECS through cargo xtask fuzz"]
fn fuzz_scene_for_the_budget() {
    let secs: u64 = std::env::var("CLATTERTRAP_FUZZ_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let deadline = Instant::now() + Duration::from_secs(secs);
    let levels: Vec<Level> = (0..ALL.len()).map(level).collect();
    let mut rounds = 0u64;
    while Instant::now() < deadline {
        // A runner stops for good once it has met its case count, so each round gets a new one.
        let mut runner = TestRunner::new(Config {
            cases: 256,
            ..Config::default()
        });
        runner
            .run(&mutated_reference(), |(i, json)| {
                holds(&levels[i], &json);
                Ok(())
            })
            .unwrap();
        runner
            .run(&scene_shaped(), |(i, json)| {
                holds(&levels[i], &json);
                Ok(())
            })
            .unwrap();
        runner
            .run(&json_tree(), |tree| {
                holds(&levels[0], &tree.to_string());
                Ok(())
            })
            .unwrap();
        rounds += 1;
    }
    println!("fuzz scene: {rounds} rounds of 768 cases in {secs} s, no panic");
}
