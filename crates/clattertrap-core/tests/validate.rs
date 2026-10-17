//! Validation rejects what the rules forbid, and every message names the path, the value, the rule, and a fix.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

mod common;

use clattertrap_core::level::{Goal, Level, Zone};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::{Material, PartSpec, Scene};
use clattertrap_core::validate::{check, parse_level, validate};
use common::{ball, beam, level, placed, scene};

fn problems(level: &Level, scene: &Scene) -> Vec<String> {
    match validate(level, scene, &Rules::V1) {
        Ok(_) => vec![],
        Err(ps) => ps.iter().map(ToString::to_string).collect(),
    }
}

fn only(level: &Level, scene: &Scene) -> String {
    let ps = problems(level, scene);
    assert_eq!(ps.len(), 1, "expected one problem, got {ps:#?}");
    ps.into_iter().next().unwrap()
}

#[test]
fn a_scene_for_another_level_is_rejected() {
    let mut sc = scene(vec![]);
    sc.level = "other".to_owned();
    let msg = only(&level(vec![]), &sc);
    assert!(msg.starts_with("level: "), "{msg}");
    assert!(
        msg.contains("\"other\"") && msg.contains("\"test\""),
        "{msg}"
    );
}

#[test]
fn check_reports_a_wrong_type_at_its_json_path() {
    let json = r#"{ "level": "test", "parts": [
      { "id": "b1", "kind": "beam", "x_cm": 100, "y_cm": 100, "length_cm": "90", "angle_deg": 0 }
    ] }"#;
    let problems = check(&level(vec![]), json, &Rules::V1).unwrap_err();
    assert_eq!(problems.len(), 1, "{problems:#?}");
    let msg = problems[0].to_string();
    // The flattened part spec hides field names from the path; the line and column still point at the value.
    assert!(msg.starts_with("parts[0]: "), "{msg}");
    assert!(msg.contains("line 2 column"), "{msg}");
    assert!(
        msg.contains("expected f32") && msg.contains("\"90\""),
        "{msg}"
    );
}

#[test]
fn check_reports_broken_json_at_the_scene_root() {
    let problems = check(&level(vec![]), "{", &Rules::V1).unwrap_err();
    assert_eq!(problems.len(), 1, "{problems:#?}");
    let msg = problems[0].to_string();
    assert!(msg.starts_with("scene: "), "{msg}");
    assert!(msg.contains("line 1"), "{msg}");
}

#[test]
fn check_on_valid_json_equals_validate() {
    let lv = level(vec![]);
    let sc = scene(vec![ball("k1", 100.0, 100.0, Material::Wood)]);
    let json = serde_json::to_string(&sc).unwrap();
    assert_eq!(
        check(&lv, &json, &Rules::V1),
        validate(&lv, &sc, &Rules::V1)
    );
}

#[test]
fn parse_level_names_the_missing_field() {
    let json = serde_json::to_string(&level(vec![]))
        .unwrap()
        .replace("\"tier\":1,", "");
    let msg = parse_level(&json).unwrap_err().to_string();
    assert!(msg.starts_with("level: "), "{msg}");
    assert!(msg.contains("tier"), "{msg}");
    assert_eq!(
        parse_level(&serde_json::to_string(&level(vec![])).unwrap()).unwrap(),
        level(vec![])
    );
}

#[test]
fn a_tier_outside_1_to_5_is_rejected() {
    for tier in [0u8, 6] {
        let mut lv = level(vec![]);
        lv.tier = tier;
        let msg = only(&lv, &scene(vec![]));
        assert!(msg.starts_with("level: "), "{msg}");
        assert!(
            msg.contains(&format!("tier {tier}")) && msg.contains("1..5"),
            "{msg}"
        );
    }
}

#[test]
fn inventory_overrun_names_the_part_past_the_limit() {
    let mut lv = level(vec![]);
    lv.inventory.beam = 2;
    let parts = (0..3)
        .map(|i| beam(&format!("b{i}"), 100.0 + 60.0 * i as f32, 100.0, 40.0, 0.0))
        .collect();
    let msg = only(&lv, &scene(parts));
    assert!(msg.starts_with("parts[2] (beam b2): "), "{msg}");
    assert!(msg.contains("2 beam") && msg.contains("remove"), "{msg}");
}

#[test]
fn a_part_centre_outside_every_zone_is_rejected_with_the_zone_bounds() {
    let mut lv = level(vec![]);
    lv.zones = vec![Zone {
        x_cm: 30.0,
        y_cm: 0.0,
        width_cm: 310.0,
        height_cm: 300.0,
    }];
    let msg = only(&lv, &scene(vec![ball("k1", 10.0, 290.0, Material::Steel)]));
    assert!(msg.starts_with("parts[0] (ball k1): "), "{msg}");
    assert!(
        msg.contains("(10, 290)") && msg.contains("outside every build zone"),
        "{msg}"
    );
    assert!(
        msg.contains("x 30..340") && msg.contains("y 0..300"),
        "{msg}"
    );
}

#[test]
fn fixed_objects_may_sit_outside_the_zones() {
    let mut lv = level(vec![ball("k1", 10.0, 290.0, Material::Steel)]);
    lv.zones = vec![Zone {
        x_cm: 30.0,
        y_cm: 0.0,
        width_cm: 310.0,
        height_cm: 300.0,
    }];
    assert!(problems(&lv, &scene(vec![])).is_empty());
}

#[test]
fn duplicate_ids_are_rejected_across_fixed_and_placed_parts() {
    let lv = level(vec![ball("k1", 50.0, 250.0, Material::Steel)]);
    let msg = only(&lv, &scene(vec![beam("k1", 200.0, 100.0, 80.0, 0.0)]));
    assert!(
        msg.starts_with("parts[0] (beam k1): ") && msg.contains("already used"),
        "{msg}"
    );
}

#[test]
fn goal_objects_in_a_scene_are_rejected() {
    let bell = placed(
        "b",
        PartSpec::Bell {
            x_cm: 200.0,
            y_cm: 100.0,
            rope_cm: 50.0,
        },
    );
    let msg = only(&level(vec![]), &scene(vec![bell]));
    assert!(
        msg.starts_with("parts[0] (bell b): ") && msg.contains("only the level may place"),
        "{msg}"
    );
}

#[test]
fn a_via_waypoint_must_name_a_part() {
    let mut lv = level(vec![]);
    lv.goal = Goal::Ring {
        target: "bell".to_owned(),
        via: vec!["nope".to_owned()],
    };
    let msg = only(&lv, &scene(vec![]));
    assert!(
        msg.starts_with("goal.via[0]: ")
            && msg.contains("\"nope\"")
            && msg.contains("names no part"),
        "{msg}"
    );
}

#[test]
fn a_beam_longer_than_the_rules_allow_is_rejected_with_the_range() {
    let msg = only(
        &level(vec![]),
        &scene(vec![beam("b1", 200.0, 100.0, 500.0, 0.0)]),
    );
    assert!(msg.starts_with("parts[0] (beam b1): "), "{msg}");
    assert!(
        msg.contains("length_cm 500") && msg.contains("20..150"),
        "{msg}"
    );
}

#[test]
fn a_negative_length_is_rejected() {
    let msg = only(
        &level(vec![]),
        &scene(vec![beam("b1", 200.0, 100.0, -5.0, 0.0)]),
    );
    assert!(
        msg.contains("length_cm -5") && msg.contains("20..150"),
        "{msg}"
    );
}

#[test]
fn parts_overlapping_at_the_start_are_rejected_with_the_depth_and_a_fix() {
    let lv = level(vec![ball("k1", 200.0, 100.0, Material::Steel)]);
    let msg = only(&lv, &scene(vec![beam("b1", 200.0, 100.0, 80.0, 0.0)]));
    assert!(msg.starts_with("parts[0] (beam b1): "), "{msg}");
    assert!(
        msg.contains("overlaps k1 by") && msg.contains("cm at the start"),
        "{msg}"
    );
    assert!(msg.contains("move it up to y_cm ="), "{msg}");
}

#[test]
fn a_part_below_the_floor_overlaps_the_floor() {
    let msg = only(
        &level(vec![]),
        &scene(vec![ball("k1", 200.0, 2.0, Material::Steel)]),
    );
    assert!(
        msg.contains("overlaps floor by") && msg.contains("move it up to y_cm = 6"),
        "{msg}"
    );
}

#[test]
fn a_ball_resting_just_above_a_beam_is_not_an_overlap() {
    let lift = (Rules::V1.ball_radius_m + Rules::V1.plank_thickness_m / 2.0) * 100.0 + 0.2;
    let lv = level(vec![ball("k1", 200.0, 100.0 + lift, Material::Steel)]);
    assert!(problems(&lv, &scene(vec![beam("b1", 200.0, 100.0, 80.0, 0.0)])).is_empty());
}

#[test]
fn every_problem_is_reported_at_once() {
    let mut lv = level(vec![]);
    lv.inventory.beam = 1;
    let parts = vec![
        beam("b1", 200.0, 100.0, 500.0, 0.0),
        beam("b1", 200.0, 100.0, 80.0, 0.0),
    ];
    let ps = problems(&lv, &scene(parts));
    assert!(ps.len() >= 3, "{ps:#?}");
}

#[test]
fn a_rope_to_a_missing_part_is_rejected() {
    let lv = level(vec![ball("k1", 50.0, 250.0, Material::Steel)]);
    let rope = placed(
        "r1",
        PartSpec::Rope {
            a: "k1".to_owned(),
            b: "zz".to_owned(),
            max_length_cm: 40.0,
        },
    );
    let msg = only(&lv, &scene(vec![rope]));
    assert!(msg.starts_with("parts[0] (rope r1): "), "{msg}");
    assert!(
        msg.contains("b = \"zz\"") && msg.contains("names no part"),
        "{msg}"
    );
}

#[test]
fn a_rope_may_not_link_a_rope() {
    let lv = level(vec![
        ball("k1", 50.0, 250.0, Material::Steel),
        ball("k2", 150.0, 250.0, Material::Steel),
    ]);
    let r1 = placed(
        "r1",
        PartSpec::Rope {
            a: "k1".to_owned(),
            b: "k2".to_owned(),
            max_length_cm: 40.0,
        },
    );
    let r2 = placed(
        "r2",
        PartSpec::Rope {
            a: "k1".to_owned(),
            b: "r1".to_owned(),
            max_length_cm: 40.0,
        },
    );
    let msg = only(&lv, &scene(vec![r1, r2]));
    assert!(
        msg.starts_with("parts[1] (rope r2): ") && msg.contains("is a rope"),
        "{msg}"
    );
}

#[test]
fn the_goal_target_must_be_a_fixed_object_of_the_right_kind() {
    let mut lv = level(vec![]);
    lv.fixed.clear();
    let msg = only(&lv, &scene(vec![]));
    assert!(
        msg.starts_with("goal: ")
            && msg.contains("\"bell\"")
            && msg.contains("names no fixed bell"),
        "{msg}"
    );

    let mut lv = level(vec![]);
    lv.goal = Goal::Press {
        target: "bell".to_owned(),
        via: vec![],
    };
    let msg = only(&lv, &scene(vec![]));
    assert!(
        msg.starts_with("goal: ") && msg.contains("names no fixed button"),
        "{msg}"
    );
}
