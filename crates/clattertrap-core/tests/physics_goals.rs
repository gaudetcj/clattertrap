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
fn a_beam_pushed_into_the_bell_at_the_start_does_not_ring_it() {
    // The beam's corner sits 2 mm inside the bell, within the overlap tolerance; the solver's push-out
    // against a static body is a large impulse, but a static part is not a blow.
    let lv = level_with_goal(vec![bell_at(360.0, 100.0)], ring(&[]));
    let r = finished(&lv, &scene(vec![beam("drop", 330.0, 128.0, 70.0, -28.0)]));
    assert!(
        !r.outcome.goal_met,
        "a static beam rang the bell at {} s",
        r.outcome.time_s
    );
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

fn via_count(r: &Run) -> usize {
    r.events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Via(_)))
        .count()
}

#[test]
fn a_waypoint_touched_by_a_separate_chain_does_not_count() {
    // The level's ball lands on the first post; a parked wood ball drops onto the second post later.
    // Nothing links the two, so the second touch belongs to a chain that never did the first post.
    let lv = level_with_goal(
        vec![
            ball("k", 100.0, 200.0, Material::Steel),
            crate_box("p1", 100.0, 15.0, 30.0),
            crate_box("p2", 300.0, 15.0, 30.0),
            bell_at(390.0, 280.0),
        ],
        ring(&["p1", "p2"]),
    );
    let sc = scene(vec![ball("lane", 300.0, 250.0, Material::Wood)]);
    let r = finished(&lv, &sc);
    assert_eq!(
        via_count(&r),
        1,
        "only p1 is touched by the chain that started on it"
    );
    assert!(!r.outcome.goal_met);
}

#[test]
fn a_part_the_chain_set_moving_touches_the_next_waypoint() {
    // The ball drops on one arm of a seesaw; the other arm swings up into the post.
    let lv = level_with_goal(
        vec![
            ball("k", 100.0, 200.0, Material::Steel),
            beam("post", 205.0, 115.0, 30.0, 0.0),
            bell_at(390.0, 280.0),
        ],
        ring(&["post"]),
    );
    let sc = scene(vec![seesaw("s", 130.0, 80.0, 150.0, -11.5)]);
    let r = finished(&lv, &sc);
    assert_eq!(
        via_count(&r),
        1,
        "the plank the ball set moving touched the post"
    );
}

#[test]
fn a_rope_joins_two_parts_into_one_chain() {
    // The falling ball yanks a wood ball off its ledge by rope; the wood ball lands on the post.
    let lv = level_with_goal(
        vec![
            ball("k", 230.0, 100.0, Material::Steel),
            beam("post", 130.0, 60.0, 140.0, 0.0),
            bell_at(390.0, 280.0),
        ],
        ring(&["post"]),
    );
    let sc = scene(vec![
        beam("shelf", 100.0, 122.0, 20.0, 0.0),
        ball("w", 100.0, 130.0, Material::Wood),
        common::rope("r", "k", "w", 150.0),
    ]);
    let r = finished(&lv, &sc);
    assert_eq!(via_count(&r), 1, "the roped wood ball touched the post");
    let k_hit_post = r
        .events
        .iter()
        .any(|e| e.kind == EventKind::contact(PartId(0), PartId(1)));
    assert!(
        !k_hit_post,
        "the steel ball itself stayed clear of the post"
    );
}

#[test]
fn a_parked_object_does_not_fill_a_basket_before_its_chain_is_done() {
    // The ball rolls off the post into the basket. A domino parked on a perch inside the basket,
    // out of the ball's way, would make two, but it never touched the post, so only the ball counts.
    let lv = level_with_goal(
        vec![
            ball("k", 54.0, 200.0, Material::Steel),
            beam("post", 62.0, 60.0, 30.0, -20.0),
            beam("perch", 132.0, 30.0, 20.0, 0.0),
            placed(
                "basket",
                PartSpec::Basket {
                    x_cm: 100.0,
                    y_cm: 0.5,
                    inner_width_cm: 60.0,
                    inner_height_cm: 40.0,
                },
            ),
        ],
        Goal::Fill {
            target: "basket".to_owned(),
            count: 2,
            via: vec!["post".to_owned()],
        },
    );
    let sc = scene(vec![common::domino("d", 127.0, 42.0)]);
    let r = finished(&lv, &sc);
    assert_eq!(via_count(&r), 1, "the ball touched the post");
    let k_rest = r
        .events
        .iter()
        .any(|e| e.kind == EventKind::AtRest(PartId(0)));
    assert!(k_rest, "the ball came to rest");
    assert!(!r.outcome.goal_met, "the parked domino must not count");
}

fn button_at(x_cm: f32, y_cm: f32) -> clattertrap_core::scene::Placed {
    placed(
        "btn",
        PartSpec::Button {
            x_cm,
            y_cm,
            travel_cm: 3.0,
        },
    )
}

fn press() -> Goal {
    Goal::Press {
        target: "btn".to_owned(),
        via: vec![],
    }
}

#[test]
fn a_crate_the_level_ball_never_met_does_not_press_the_button() {
    // The level's ball falls to the floor far from the button; the crate alone sits on the cap.
    let lv = level_with_goal(
        vec![
            ball("ball0", 80.0, 250.0, Material::Steel),
            button_at(330.0, 40.0),
        ],
        press(),
    );
    let r = finished(&lv, &scene(vec![crate_box("c", 330.0, 60.0, 20.0)]));
    assert!(
        !r.outcome.goal_met,
        "a crate placed on the cap pressed it at {} s",
        r.outcome.time_s
    );
}

#[test]
fn a_crate_the_level_ball_lands_on_presses_the_button() {
    let lv = level_with_goal(
        vec![
            ball("ball0", 330.0, 200.0, Material::Steel),
            button_at(330.0, 40.0),
        ],
        press(),
    );
    let r = finished(&lv, &scene(vec![crate_box("c", 330.0, 60.0, 20.0)]));
    assert!(
        r.outcome.goal_met,
        "the ball landing on the crate did not count"
    );
    assert!(
        r.outcome.time_s > 0.3,
        "pressed at {} s, before the ball arrived",
        r.outcome.time_s
    );
}

#[test]
fn a_placed_ball_the_level_ball_never_met_does_not_ring_the_bell() {
    let lv = level_with_goal(
        vec![
            ball("ball0", 50.0, 250.0, Material::Steel),
            bell_at(200.0, 100.0),
        ],
        ring(&[]),
    );
    let r = finished(&lv, &scene(vec![ball("k", 200.0, 200.0, Material::Steel)]));
    assert!(
        !r.outcome.goal_met,
        "a placed ball rang the bell on its own at {} s",
        r.outcome.time_s
    );
}
