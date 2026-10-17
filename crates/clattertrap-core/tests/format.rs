//! Level and scene JSON: the formats models read and write.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use clattertrap_core::level::{Goal, Level};
use clattertrap_core::parts::{Look, Visual};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::{Material, PartSpec, Scene};

const BELL_LEVEL: &str = r#"{
  "id": "bell-drop",
  "title": "Ring the bell",
  "tier": 1,
  "description": "A steel ball falls from the top left. The bell hangs on the right.",
  "rules_version": 1,
  "world": { "width_cm": 400, "height_cm": 300 },
  "time_limit_s": 20,
  "fixed": [
    { "id": "ball0", "kind": "ball", "material": "steel", "x_cm": 70, "y_cm": 280 },
    { "id": "bell", "kind": "bell", "x_cm": 360, "y_cm": 100, "rope_cm": 50 }
  ],
  "zones": [ { "x_cm": 30, "y_cm": 0, "width_cm": 310, "height_cm": 300 } ],
  "inventory": { "beam": 4, "seesaw": 1, "domino": 6 },
  "goal": { "kind": "ring", "target": "bell" }
}"#;

#[test]
fn level_round_trips_through_json() {
    let level: Level = serde_json::from_str(BELL_LEVEL).unwrap();
    assert_eq!(level.id, "bell-drop");
    assert_eq!(level.fixed.len(), 2);
    assert_eq!(level.fixed[1].id, "bell");
    assert_eq!(level.inventory.beam, 4);
    assert_eq!(level.inventory.rope, 0);
    assert_eq!(
        level.goal,
        Goal::Ring {
            target: "bell".to_owned(),
            via: vec![]
        }
    );
    assert!(!level.showcase);
    assert_eq!(level.tier, 1);
    assert!(level.description.starts_with("A steel ball"));
    let again: Level = serde_json::from_str(&serde_json::to_string(&level).unwrap()).unwrap();
    assert_eq!(again, level);
}

#[test]
fn a_level_needs_a_tier_and_a_description() {
    for field in ["tier", "description"] {
        let without: String = BELL_LEVEL
            .lines()
            .filter(|l| !l.contains(&format!("\"{field}\"")))
            .collect::<Vec<_>>()
            .join("\n");
        let err = serde_json::from_str::<Level>(&without)
            .unwrap_err()
            .to_string();
        assert!(err.contains(field), "{err}");
    }
}

#[test]
fn fill_goal_carries_a_count_and_via_waypoints() {
    let json = r#"{ "kind": "fill", "target": "basket", "count": 2, "via": ["s1", "d3"] }"#;
    let goal: Goal = serde_json::from_str(json).unwrap();
    assert_eq!(goal.target(), "basket");
    assert_eq!(goal.via(), ["s1", "d3"]);
    assert_eq!(
        goal,
        Goal::Fill {
            target: "basket".to_owned(),
            count: 2,
            via: vec!["s1".to_owned(), "d3".to_owned()]
        }
    );
}

#[test]
fn scene_parses_every_part_kind() {
    let json = r#"{ "level": "x", "parts": [
      { "id": "a", "kind": "beam", "x_cm": 1, "y_cm": 2, "length_cm": 80, "angle_deg": -15 },
      { "id": "b", "kind": "funnel", "x_cm": 1, "y_cm": 2, "mouth_cm": 60, "throat_cm": 16, "height_cm": 40 },
      { "id": "c", "kind": "ball", "x_cm": 1, "y_cm": 2, "material": "rubber" },
      { "id": "d", "kind": "domino", "x_cm": 1, "y_cm": 2 },
      { "id": "e", "kind": "crate", "x_cm": 1, "y_cm": 2, "side_cm": 20 },
      { "id": "f", "kind": "seesaw", "x_cm": 1, "y_cm": 2, "length_cm": 100 },
      { "id": "g", "kind": "pendulum", "x_cm": 1, "y_cm": 2, "length_cm": 50, "angle_deg": 30 },
      { "id": "h", "kind": "spring_pad", "x_cm": 1, "y_cm": 2 },
      { "id": "i", "kind": "bucket", "x_cm": 1, "y_cm": 2, "width_cm": 30, "height_cm": 25 },
      { "id": "j", "kind": "balloon", "x_cm": 1, "y_cm": 2 },
      { "id": "k", "kind": "pin", "x_cm": 1, "y_cm": 2 },
      { "id": "l", "kind": "rope", "a": "c", "b": "e", "max_length_cm": 40 },
      { "id": "m", "kind": "bell", "x_cm": 1, "y_cm": 2, "rope_cm": 50 },
      { "id": "n", "kind": "button", "x_cm": 1, "y_cm": 2, "travel_cm": 3 },
      { "id": "o", "kind": "basket", "x_cm": 1, "y_cm": 2, "inner_width_cm": 50, "inner_height_cm": 40 }
    ] }"#;
    let scene: Scene = serde_json::from_str(json).unwrap();
    let kinds: Vec<&str> = scene.parts.iter().map(|p| p.spec.kind()).collect();
    assert_eq!(
        kinds,
        [
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
            "basket"
        ]
    );
    assert!(matches!(
        scene.parts[2].spec,
        PartSpec::Ball {
            material: Material::Rubber,
            ..
        }
    ));
    assert!(
        matches!(scene.parts[3].spec, PartSpec::Domino { angle_deg, .. } if angle_deg.abs() < 1e-6)
    );
    assert!(
        matches!(scene.parts[10].spec, PartSpec::Pin { angle_deg, .. } if (angle_deg + 90.0).abs() < 1e-6)
    );
    assert!(matches!(&scene.parts[11].spec, PartSpec::Rope { a, b, .. } if a == "c" && b == "e"));
}

#[test]
fn unknown_part_field_is_rejected() {
    let json = r#"{ "level": "x", "parts": [ { "id": "a", "kind": "ball", "x_cm": 1, "y_cm": 2, "material": "steel", "radius_cm": 9 } ] }"#;
    let err = serde_json::from_str::<Scene>(json).unwrap_err().to_string();
    assert!(err.contains("radius_cm"), "{err}");
}

#[test]
fn unknown_part_kind_is_rejected() {
    let json = r#"{ "level": "x", "parts": [ { "id": "a", "kind": "trampoline", "x_cm": 1, "y_cm": 2 } ] }"#;
    let err = serde_json::from_str::<Scene>(json).unwrap_err().to_string();
    assert!(err.contains("trampoline"), "{err}");
}

#[test]
fn missing_required_field_is_rejected() {
    let json = r#"{ "level": "x", "parts": [ { "id": "a", "kind": "beam", "x_cm": 1, "y_cm": 2, "angle_deg": 0 } ] }"#;
    let err = serde_json::from_str::<Scene>(json).unwrap_err().to_string();
    assert!(err.contains("length_cm"), "{err}");
}

#[test]
fn outcome_serializes_hashes_as_hex_strings() {
    let level: Level = serde_json::from_str(BELL_LEVEL).unwrap();
    let scene = Scene {
        level: "bell-drop".to_owned(),
        parts: vec![],
    };
    let valid = clattertrap_core::validate::validate(&level, &scene, &Rules::V1).unwrap();
    let outcome = clattertrap_core::sim::run(&level, &valid, &Rules::V1).outcome;
    let json = serde_json::to_value(&outcome).unwrap();
    assert_eq!(
        json["event_hash"],
        serde_json::Value::String(format!("{:016x}", outcome.event_hash))
    );
    assert_eq!(
        json["state_hash"],
        serde_json::Value::String(format!("{:016x}", outcome.state_hash))
    );
    assert_eq!(json["state_hash"].as_str().unwrap().len(), 16);
    assert_eq!(json["end"], "all_asleep");
    assert_eq!(json["goal_met"], false);
    assert_eq!(json["rules_version"], 1);
}

#[test]
fn visuals_serialize_with_a_shape_tag() {
    let circle = Visual::Circle {
        r_m: 0.06,
        look: Look::Rubber,
    };
    assert_eq!(
        serde_json::to_string(&circle).unwrap(),
        r#"{"shape":"circle","r_m":0.06,"look":"rubber"}"#
    );
    assert_eq!(
        serde_json::to_string(&Visual::Pivot).unwrap(),
        r#"{"shape":"pivot"}"#
    );
    assert_eq!(serde_json::to_string(&Look::Pad).unwrap(), r#""pad""#);
}

#[test]
fn the_scene_schema_names_every_part_kind_and_closes_objects() {
    let text = serde_json::to_string_pretty(&clattertrap_core::scene::schema()).unwrap();
    for kind in clattertrap_core::parts::SUPPORTED {
        assert!(text.contains(&format!("\"{kind}\"")), "{kind} missing");
    }
    assert!(text.contains("\"additionalProperties\": false"), "{text}");
    assert!(text.contains("\"max_length_cm\""), "{text}");
    // insta needs the snapshot file, which wasmtime cannot see; the asserts above run everywhere.
    #[cfg(not(target_family = "wasm"))]
    insta::assert_snapshot!(text);
}

#[test]
fn the_level_schema_carries_the_goal_kinds_and_the_tier() {
    let text = serde_json::to_string_pretty(&clattertrap_core::level::schema()).unwrap();
    for kind in ["ring", "press", "fill", "pop"] {
        assert!(text.contains(&format!("\"{kind}\"")), "{kind} missing");
    }
    assert!(
        text.contains("\"tier\"") && text.contains("\"description\""),
        "{text}"
    );
    // insta needs the snapshot file, which wasmtime cannot see; the asserts above run everywhere.
    #[cfg(not(target_family = "wasm"))]
    insta::assert_snapshot!(text);
}
