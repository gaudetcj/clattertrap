//! Spring pads launch, balloons rise and pop on pins, funnels guide, buckets hold.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

mod common;

use clattertrap_core::events::EventKind;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use common::{
    ball, balloon, bucket, close, domino, funnel, level, part, pin, portal, scene, sim, spring_pad,
    step_for,
};

#[test]
fn a_spring_pad_launches_a_wood_ball_faster_than_it_arrived() {
    let lv = level(vec![ball("k", 200.0, 120.0, Material::Wood)]);
    let mut s = sim(&lv, &scene(vec![spring_pad("pad", 200.0, 20.0, 0.0)]));
    let k = part(&s, "k");
    let (mut arrival, mut launch, mut apex) = (0.0f32, 0.0f32, 0.0f32);
    let mut launched = false;
    while s.time_s() < 3.0 && s.finished().is_none() {
        s.step();
        let Some(v) = s.velocity(k) else { break };
        let vy = v[1];
        if !launched && vy < 0.0 {
            arrival = arrival.max(-vy);
        }
        if vy > 0.0 {
            launched = true;
            launch = launch.max(vy);
        }
        if launched {
            apex = apex.max(s.pose(k).map_or(apex, |p| p.y_m));
        }
    }
    assert!(launched, "the ball never left the pad");
    assert!(
        launch > arrival,
        "left at {launch} m/s after arriving at {arrival} m/s"
    );
    assert!(apex > 1.2, "apex after the pad was only {apex} m");
}

#[test]
fn a_balloon_rises_at_a_steady_speed() {
    let lv = level(vec![balloon("bl", 200.0, 50.0)]);
    let mut s = sim(&lv, &scene(vec![]));
    let b = part(&s, "bl");
    let mut last_y = s.pose(b).unwrap().y_m;
    step_for(&mut s, 1.5);
    let r = Rules::V1;
    let expected = -r.balloon_gravity_scale * r.gravity_m_s2 / r.balloon_damping;
    let vy = s.velocity(b).unwrap()[1];
    assert!(
        close(vy, expected, 0.20),
        "rising at {vy} m/s, expected about {expected} m/s"
    );
    for _ in 0..60 {
        s.step();
        let y = s.pose(b).unwrap().y_m;
        assert!(y > last_y, "balloon stopped rising at {y} m");
        last_y = y;
    }
}

#[test]
fn a_balloon_pops_on_a_pin_and_disappears() {
    let lv = level(vec![
        balloon("bl", 200.0, 50.0),
        pin("needle", 200.0, 290.0),
    ]);
    let mut s = sim(&lv, &scene(vec![]));
    let (b, n) = (part(&s, "bl"), part(&s, "needle"));
    let mut popped_at = None;
    while s.time_s() < 5.0 && popped_at.is_none() {
        if s.step().iter().any(|e| e.kind == EventKind::contact(b, n)) {
            popped_at = Some(s.time_s());
        }
    }
    let t = popped_at.expect("the balloon should reach the pin");
    assert!(t > 1.0 && t < 3.0, "popped at {t} s");
    assert!(s.pose(b).is_none(), "the balloon body should be gone");
    assert!(s.slots().all(|slot| slot.part != b || slot.pose.is_none()));
    step_for(&mut s, 0.5);
    assert!(s.poses().all(|p| p.x_m.is_finite() && p.y_m.is_finite()));
}

#[test]
fn a_funnel_guides_an_off_centre_ball_through_its_throat() {
    let lv = level(vec![ball("k", 170.0, 220.0, Material::Wood)]);
    let mut s = sim(
        &lv,
        &scene(vec![funnel("f", 200.0, 100.0, 100.0, 16.0, 60.0)]),
    );
    let k = part(&s, "k");
    let mut passed_throat = false;
    while s.time_s() < 4.0 && s.finished().is_none() {
        s.step();
        let p = s.pose(k).unwrap();
        if p.y_m < 1.0 && (p.x_m - 2.0).abs() < 0.08 {
            passed_throat = true;
        }
    }
    assert!(passed_throat, "the ball never came out of the throat");
    let p = s.pose(k).unwrap();
    assert!(
        (p.x_m - 2.0).abs() < 0.3 && p.y_m < 0.1,
        "ball ended at ({}, {})",
        p.x_m,
        p.y_m
    );
}

#[test]
fn a_bucket_catches_a_ball_and_stays_upright() {
    let lv = level(vec![ball("k", 205.0, 150.0, Material::Wood)]);
    let mut s = sim(&lv, &scene(vec![bucket("bk", 200.0, 0.5, 30.0, 25.0)]));
    let (k, bk) = (part(&s, "k"), part(&s, "bk"));
    step_for(&mut s, 3.0);
    let ball_at = s.pose(k).unwrap();
    assert!(
        (ball_at.x_m - 2.0).abs() < 0.15 && ball_at.y_m < 0.30,
        "ball ended at ({}, {})",
        ball_at.x_m,
        ball_at.y_m
    );
    let bucket_at = s.pose(bk).unwrap();
    assert!(
        bucket_at.angle_rad.abs() < 0.1,
        "bucket tipped {} rad",
        bucket_at.angle_rad
    );
}

#[test]
fn a_ball_dropped_into_a_floor_portal_leaves_the_wall_portal_at_the_same_speed() {
    // Orange end in the floor under the ball, blue end low on the left wall facing right.
    let lv = level(vec![
        ball("k", 200.0, 150.0, Material::Steel),
        portal("p", [200.0, 3.0, 90.0], [10.0, 200.0, 0.0]),
    ]);
    let mut s = sim(&lv, &scene(vec![]));
    let k = part(&s, "k");
    let mut entry_speed = 0.0f32;
    let mut exit: Option<([f32; 2], [f32; 2])> = None;
    while s.time_s() < 2.0 && s.finished().is_none() {
        s.step();
        let (Some(p), Some(v)) = (s.pose(k), s.velocity(k)) else {
            break;
        };
        if exit.is_none() && p.x_m > 1.0 {
            entry_speed = libm::sqrtf(v[0] * v[0] + v[1] * v[1]);
        } else if exit.is_none() {
            exit = Some(([p.x_m, p.y_m], v));
        }
    }
    let (at, v) = exit.expect("the ball never came out of the blue end");
    assert!(
        at[0] < 0.3 && close(at[1], 2.0, 0.05),
        "came out at ({}, {}) m",
        at[0],
        at[1]
    );
    assert!(entry_speed > 4.0, "entered at only {entry_speed} m/s");
    let exit_speed = libm::sqrtf(v[0] * v[0] + v[1] * v[1]);
    assert!(
        close(exit_speed, entry_speed, 0.05 * entry_speed),
        "left at {exit_speed} m/s after entering at {entry_speed} m/s"
    );
    assert!(
        v[0] > 0.95 * exit_speed,
        "left heading ({}, {}) instead of out along the blue end's facing",
        v[0],
        v[1]
    );
}

#[test]
fn a_domino_falls_through_a_portal_as_if_it_were_not_there() {
    let lv = level(vec![portal("p", [200.0, 3.0, 90.0], [10.0, 200.0, 0.0])]);
    let mut s = sim(&lv, &scene(vec![domino("d", 200.0, 40.0)]));
    let d = part(&s, "d");
    step_for(&mut s, 1.5);
    let p = s.pose(d).expect("the domino is still there");
    assert!(
        close(p.x_m, 2.0, 0.05) && p.y_m < 0.15,
        "the domino ended at ({}, {}) m",
        p.x_m,
        p.y_m
    );
}
