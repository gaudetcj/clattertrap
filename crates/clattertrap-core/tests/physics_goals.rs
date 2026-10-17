//! Goals fire for the right reasons, waypoints gate them, and runs end the way the rules say.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

mod common;

use clattertrap_core::events::{EventKind, PartId};
use clattertrap_core::level::Goal;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::{Material, PartSpec, Scene};
use clattertrap_core::score::End;
use clattertrap_core::sim::{Run, run};
use clattertrap_core::validate::validate;
use common::{
    ball, balloon, beam, crate_box, level_with_goal, pendulum, pin, placed, scene, seesaw,
};

fn bell_at(x_cm: f32, y_cm: f32) -> clattertrap_core::scene::Placed {
    placed(
        "bell",
        PartSpec::Bell {
            x_cm,
            y_cm,
            rope_cm: 50.0,
        },
    )
}

fn ring(via: &[&str]) -> Goal {
    Goal::Ring {
        target: "bell".to_owned(),
        via: via.iter().map(|s| (*s).to_owned()).collect(),
    }
}

fn finished(level: &clattertrap_core::level::Level, sc: &Scene) -> Run {
    let valid = validate(level, sc, &Rules::V1).unwrap_or_else(|p| panic!("{p:#?}"));
    run(level, &valid, &Rules::V1)
}

#[test]
fn a_dropped_steel_ball_rings_the_bell_and_ends_the_run() {
    let lv = level_with_goal(
        vec![
            ball("k", 200.0, 200.0, Material::Steel),
            bell_at(200.0, 100.0),
        ],
        ring(&[]),
    );
    let r = finished(&lv, &scene(vec![]));
    assert!(r.outcome.goal_met);
    assert_eq!(r.outcome.end, End::GoalMet);
    assert!(
        r.outcome.time_s > 0.3 && r.outcome.time_s < 1.0,
        "rang at {} s",
        r.outcome.time_s
    );
    let last = r.events.last().unwrap();
    assert_eq!(last.kind, EventKind::GoalMet);
    assert_eq!(last.step, r.outcome.steps);
}

#[test]
fn a_feather_touch_does_not_ring_the_bell() {
    let r = Rules::V1;
    let y = 100.0 + (r.bell_radius_m + r.ball_radius_m) * 100.0 + 0.5;
    let lv = level_with_goal(
        vec![ball("k", 200.0, y, Material::Wood), bell_at(200.0, 100.0)],
        ring(&[]),
    );
    let out = finished(&lv, &scene(vec![])).outcome;
    assert!(!out.goal_met, "a 5 mm drop rang the bell");
    assert_ne!(out.end, End::GoalMet);
}

#[test]
fn waypoints_must_be_touched_before_the_goal_counts() {
    let touched = level_with_goal(
        vec![
            ball("k", 100.0, 200.0, Material::Steel),
            bell_at(260.0, 100.0),
        ],
        ring(&["ramp"]),
    );
    let sc = scene(vec![
        beam("ramp", 140.0, 160.0, 120.0, -25.0),
        beam("other", 300.0, 250.0, 40.0, 0.0),
    ]);
    let r = finished(&touched, &sc);
    assert!(
        r.outcome.goal_met,
        "ball should roll down the ramp into the bell"
    );
    let via_step = r
        .events
        .iter()
        .find(|e| matches!(e.kind, EventKind::Via(_)))
        .expect("a Via event")
        .step;
    assert!(via_step < r.outcome.steps);

    let skipped = level_with_goal(
        vec![
            ball("k", 100.0, 200.0, Material::Steel),
            bell_at(260.0, 100.0),
        ],
        ring(&["other"]),
    );
    let r = finished(&skipped, &sc);
    assert!(
        !r.outcome.goal_met,
        "the bell rang without touching the waypoint"
    );
    assert!(!r.events.iter().any(|e| e.kind == EventKind::GoalMet));
}

#[test]
fn a_crate_presses_the_button() {
    let button = placed(
        "btn",
        PartSpec::Button {
            x_cm: 200.0,
            y_cm: 10.0,
            travel_cm: 3.0,
        },
    );
    let goal = Goal::Press {
        target: "btn".to_owned(),
        via: vec![],
    };
    let pressed = level_with_goal(vec![button.clone()], goal.clone());
    let r = finished(&pressed, &scene(vec![crate_box("c", 200.0, 60.0, 20.0)]));
    assert!(
        r.outcome.goal_met,
        "a falling crate should press the button"
    );
    assert_eq!(r.outcome.end, End::GoalMet);

    let untouched = level_with_goal(vec![button], goal);
    let r = finished(&untouched, &scene(vec![]));
    assert!(!r.outcome.goal_met, "an untouched button pressed itself");
    assert_eq!(r.outcome.end, End::AllAsleep);
}

#[test]
fn two_balls_at_rest_in_the_basket_fill_it_and_one_does_not() {
    let basket = placed(
        "bk",
        PartSpec::Basket {
            x_cm: 200.0,
            y_cm: 0.5,
            inner_width_cm: 50.0,
            inner_height_cm: 40.0,
        },
    );
    let goal = Goal::Fill {
        target: "bk".to_owned(),
        count: 2,
        via: vec![],
    };
    let two = level_with_goal(
        vec![
            basket.clone(),
            ball("a", 190.0, 120.0, Material::Wood),
            ball("b", 210.0, 150.0, Material::Wood),
        ],
        goal.clone(),
    );
    let r = finished(&two, &scene(vec![]));
    assert!(r.outcome.goal_met, "two balls should fill the basket");
    assert!(
        r.outcome.time_s > 0.4,
        "filled before the balls could land, at {} s",
        r.outcome.time_s
    );

    let one = level_with_goal(vec![basket, ball("a", 190.0, 120.0, Material::Wood)], goal);
    let r = finished(&one, &scene(vec![]));
    assert!(!r.outcome.goal_met);
    assert_eq!(r.outcome.end, End::AllAsleep);
}

#[test]
fn popping_the_target_balloon_meets_a_pop_goal() {
    let goal = Goal::Pop {
        target: "bl".to_owned(),
        via: vec![],
    };
    let lv = level_with_goal(
        vec![balloon("bl", 200.0, 50.0), pin("needle", 200.0, 290.0)],
        goal,
    );
    let r = finished(&lv, &scene(vec![]));
    assert!(r.outcome.goal_met);
    assert_eq!(r.outcome.end, End::GoalMet);
}

#[test]
fn a_body_that_falls_out_of_the_world_is_removed_with_an_event() {
    let lv = level_with_goal(
        vec![
            ball("k", -30.0, 100.0, Material::Steel),
            bell_at(380.0, 200.0),
        ],
        ring(&[]),
    );
    let r = finished(&lv, &scene(vec![]));
    let left = r
        .events
        .iter()
        .find(|e| matches!(e.kind, EventKind::LeftWorld(_)))
        .expect("a LeftWorld event");
    assert!(left.step > 0);
    assert_eq!(r.outcome.end, End::AllAsleep);
    assert!(!r.outcome.goal_met);
}

#[test]
fn a_run_ends_at_the_time_limit_while_something_still_moves() {
    let mut lv = level_with_goal(vec![bell_at(380.0, 200.0)], ring(&[]));
    lv.time_limit_s = 0.5;
    let r = finished(&lv, &scene(vec![pendulum("p", 200.0, 250.0, 100.0, 40.0)]));
    assert_eq!(r.outcome.end, End::TimeLimit);
    assert_eq!(r.outcome.steps, 60);
    assert!((r.outcome.time_s - 0.5).abs() < 1e-6);
}

#[test]
fn a_resting_scene_ends_asleep_after_at_rest_events() {
    let lv = level_with_goal(
        vec![
            ball("k", 200.0, 30.0, Material::Steel),
            bell_at(380.0, 200.0),
        ],
        ring(&[]),
    );
    let r = finished(&lv, &scene(vec![]));
    assert_eq!(r.outcome.end, End::AllAsleep);
    assert!(
        r.outcome.time_s < 5.0,
        "took {} s to sleep",
        r.outcome.time_s
    );
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e.kind, EventKind::AtRest(_))),
        "no AtRest event in {:?}",
        r.events
    );
}

#[test]
fn chain_length_counts_distinct_parts_set_moving() {
    let lv = level_with_goal(
        vec![
            ball("k", 160.0, 150.0, Material::Steel),
            bell_at(380.0, 250.0),
        ],
        ring(&[]),
    );
    let sc = scene(vec![
        seesaw("s", 200.0, 12.0, 100.0, -10.0),
        crate_box("c", 240.0, 13.5, 10.0),
    ]);
    let r = finished(&lv, &sc);
    assert_eq!(
        r.outcome.chain_length, 3,
        "ball, plank, and crate should all move: {:?}",
        r.outcome
    );
    assert_eq!(r.outcome.parts_used, 2);
}

#[test]
fn closing_appears_at_the_end_and_matches_run() {
    let lv = common::level(vec![ball("k", 70.0, 280.0, Material::Steel)]);
    let sc = scene(vec![beam("b1", 120.0, 210.0, 120.0, -15.0)]);
    let valid = validate(&lv, &sc, &Rules::V1).unwrap();
    let mut sim = clattertrap_core::sim::Simulator::new(&lv, &valid, &Rules::V1);
    let mut recorder = clattertrap_core::score::Recorder::new();
    assert!(sim.closing().is_none(), "no closing before the run ends");
    while sim.finished().is_none() {
        recorder.record(sim.step());
    }
    let closing = sim.closing().expect("a finished run has a closing");
    let (outcome, events) = recorder.finish(closing);
    let r = run(&lv, &valid, &Rules::V1);
    assert_eq!(outcome, r.outcome);
    assert_eq!(events, r.events);
}

#[test]
fn a_waypoint_counts_only_when_something_moving_touches_it() {
    // The post rests on the floor from the first step; only the falling ball touches it.
    let lv = level_with_goal(
        vec![
            crate_box("post", 200.0, 15.0, 30.0),
            ball("k", 200.0, 200.0, Material::Steel),
            bell_at(300.0, 100.0),
        ],
        ring(&["post"]),
    );
    let r = finished(&lv, &scene(vec![]));
    assert!(!r.outcome.goal_met);
    let via = r
        .events
        .iter()
        .find(|e| matches!(e.kind, EventKind::Via(_)))
        .expect("the ball touched the post");
    let landing = r
        .events
        .iter()
        .find(|e| e.kind == EventKind::contact(PartId(0), PartId(1)))
        .expect("the ball landed on the post");
    assert!(
        via.step > 1,
        "via fired at step {} before anything moved",
        via.step
    );
    assert_eq!(via.step, landing.step);
}
