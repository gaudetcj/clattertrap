//! Dominoes fall in order, a seesaw flings, a pendulum keeps time, and a rope holds.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::many_single_char_names
)]

mod common;

use clattertrap_core::events::EventKind;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use common::{
    PI, ball, beam, close, crate_box, domino, level, on_plank, part, pendulum, rope, scene, seesaw,
    sim, step_for,
};

#[test]
fn a_row_of_dominoes_falls_in_order() {
    let r = Rules::V1;
    let [bx, by] = on_plank(115.0, 25.0, -20.0, -25.0, r.ball_radius_m);
    let lv = level(vec![ball("k", bx, by, Material::Steel)]);
    let half_h = r.domino_size_m[1] * 50.0;
    let mut parts = vec![beam("ramp", 115.0, 25.0, 60.0, -20.0)];
    parts.extend((0..5).map(|i| domino(&format!("d{i}"), 150.0 + 8.0 * i as f32, half_h + 0.2)));
    let mut s = sim(&lv, &scene(parts));
    let ids: Vec<_> = (0..5).map(|i| part(&s, &format!("d{i}"))).collect();
    let mut first_touch = [None; 4];
    while s.time_s() < 4.0 && s.finished().is_none() {
        let step = s.step_index() + 1;
        for e in s.step() {
            for (i, slot) in first_touch.iter_mut().enumerate() {
                if e.kind == EventKind::contact(ids[i], ids[i + 1]) && slot.is_none() {
                    *slot = Some(step);
                }
            }
        }
    }
    let steps: Vec<u32> = first_touch
        .iter()
        .map(|t| t.expect("every neighbour pair should touch"))
        .collect();
    assert!(
        steps.windows(2).all(|w| w[0] < w[1]),
        "dominoes touched out of order: {steps:?}"
    );
    for id in &ids {
        let tilt = s.pose(*id).unwrap().angle_rad.abs();
        assert!(tilt > PI / 4.0, "domino {id:?} only tilted {tilt} rad");
    }
}

#[test]
fn a_seesaw_flings_a_crate_when_a_steel_ball_lands_on_the_other_end() {
    let lv = level(vec![ball("k", 160.0, 150.0, Material::Steel)]);
    let sc = scene(vec![
        seesaw("s", 200.0, 12.0, 100.0, -10.0),
        crate_box("c", 240.0, 13.5, 10.0),
    ]);
    let mut s = sim(&lv, &sc);
    let c = part(&s, "c");
    let start_y = s.pose(c).unwrap().y_m;
    let mut top = start_y;
    while s.time_s() < 3.0 {
        s.step();
        top = top.max(s.pose(c).unwrap().y_m);
    }
    assert!(top > start_y + 0.10, "crate rose only {} m", top - start_y);
}

#[test]
fn a_pendulum_swings_with_the_small_angle_period() {
    let lv = level(vec![]);
    let mut s = sim(&lv, &scene(vec![pendulum("p", 200.0, 250.0, 100.0, 10.0)]));
    let p = part(&s, "p");
    let mut crossings = Vec::new();
    let mut last_x = s.pose(p).unwrap().x_m;
    while s.time_s() < 6.0 && crossings.len() < 3 {
        s.step();
        let x = s.pose(p).unwrap().x_m;
        if last_x > 2.0 && x <= 2.0 {
            crossings.push(s.time_s());
        }
        last_x = x;
    }
    assert!(
        crossings.len() >= 2,
        "pendulum crossed the vertical only {} times",
        crossings.len()
    );
    let period = crossings[1] - crossings[0];
    let expected = 2.0 * PI * libm::sqrtf(1.0 / Rules::V1.gravity_m_s2);
    assert!(
        close(period, expected, 0.10),
        "period {period} s, expected {expected} s"
    );
}

#[test]
fn a_rope_keeps_two_balls_within_its_length() {
    let r = Rules::V1;
    let rest = 102.0 + (r.ball_radius_m * 100.0) + 0.2;
    let lv = level(vec![]);
    let sc = scene(vec![
        beam("ledge", 150.0, 100.0, 100.0, 0.0),
        ball("a", 185.0, rest, Material::Wood),
        ball("b", 215.0, rest, Material::Wood),
        rope("r", "a", "b", 50.0),
    ]);
    let mut s = sim(&lv, &sc);
    let (a, b) = (part(&s, "a"), part(&s, "b"));
    let mut longest = 0.0f32;
    while s.time_s() < 3.0 {
        s.step();
        let (pa, pb) = (s.pose(a).unwrap(), s.pose(b).unwrap());
        let (dx, dy) = (pa.x_m - pb.x_m, pa.y_m - pb.y_m);
        let d = libm::sqrtf(dx * dx + dy * dy);
        longest = longest.max(d);
    }
    assert!(
        longest > 0.45,
        "the rope never went taut; longest gap {longest} m"
    );
    assert!(longest < 0.52, "the rope stretched to {longest} m");
}

#[test]
fn a_crate_dropped_on_the_floor_comes_to_rest_upright() {
    let lv = level(vec![]);
    let mut s = sim(&lv, &scene(vec![crate_box("c", 200.0, 60.0, 20.0)]));
    let c = part(&s, "c");
    step_for(&mut s, 3.0);
    let p = s.pose(c).unwrap();
    assert!(close(p.y_m, 0.10, 0.05), "crate centre at {} m", p.y_m);
    assert!(p.angle_rad.abs() < 0.05, "crate tilted {} rad", p.angle_rad);
}

#[test]
fn a_pendulum_rod_is_stopped_by_a_beam_the_weight_would_clear() {
    // The rod sweeps through a short post near the pin; the weight itself never comes near it.
    let lv = level(vec![beam("b", 215.0, 230.0, 20.0, 90.0)]);
    let mut s = sim(&lv, &scene(vec![pendulum("p", 200.0, 250.0, 100.0, -60.0)]));
    let (p, b) = (part(&s, "p"), part(&s, "b"));
    let mut touched = false;
    let mut far_x = 0.0_f32;
    while s.time_s() < 3.0 {
        touched |= s.step().iter().any(|e| e.kind == EventKind::contact(p, b));
        far_x = far_x.max(s.pose(p).unwrap().x_m);
    }
    assert!(touched, "the rod never touched the beam");
    assert!(far_x < 2.70, "the weight swung on to x = {far_x} m");
}
