//! Free fall, rolling down an incline, and bounce heights, checked against textbook results.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod common;

use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use common::{PI, ball, beam, close, level, part, scene, sim, step_for};

#[test]
fn free_fall_matches_half_g_t_squared() {
    let lv = level(vec![ball("b", 200.0, 250.0, Material::Steel)]);
    let mut s = sim(&lv, &scene(vec![]));
    let b = part(&s, "b");
    let y0 = s.pose(b).unwrap().y_m;
    step_for(&mut s, 0.5);
    let drop = y0 - s.pose(b).unwrap().y_m;
    let expected = 0.5 * Rules::V1.gravity_m_s2 * 0.5 * 0.5;
    assert!(
        close(drop, expected, 0.03),
        "dropped {drop} m, expected {expected} m"
    );
}

#[test]
fn ball_rolls_down_a_20_degree_incline_in_the_expected_time() {
    let theta = 20.0 * PI / 180.0;
    let (cos, sin) = (libm::cosf(theta), libm::sinf(theta));
    let (cx, cy, len) = (200.0f32, 100.0f32, 150.0f32);
    let r = Rules::V1;
    // Ball resting on the plank 10 cm below the upper end.
    let along = len / 2.0 - 10.0;
    let lift = (r.ball_radius_m + r.plank_thickness_m / 2.0) * 100.0 + 0.2;
    let (bx, by) = (cx + along * cos - lift * sin, cy + along * sin + lift * cos);
    let lv = level(vec![ball("b", bx, by, Material::Wood)]);
    let mut s = sim(&lv, &scene(vec![beam("ramp", cx, cy, len, 20.0)]));
    let b = part(&s, "b");
    let start = s.pose(b).unwrap();
    let mut travelled = 0.0;
    while travelled < 1.0 {
        s.step();
        assert!(
            s.time_s() < 3.0,
            "ball never rolled 1 m; travelled {travelled} m"
        );
        let p = s.pose(b).unwrap();
        travelled = (start.x_m - p.x_m) * cos + (start.y_m - p.y_m) * sin;
    }
    // Solid disc: a = g sin(theta) / (1 + 1/2), see docs/physics.md#rolling.
    let accel = r.gravity_m_s2 * sin / 1.5;
    let expected_t = libm::sqrtf(2.0 * 1.0 / accel);
    assert!(
        close(s.time_s(), expected_t, 0.10),
        "took {} s, expected {expected_t} s",
        s.time_s()
    );
    let v = s.velocity(b).unwrap();
    let speed = libm::sqrtf(v[0] * v[0] + v[1] * v[1]);
    let rim = s.angular_velocity(b).unwrap().abs() * r.ball_radius_m;
    assert!(
        close(rim, speed, 0.10),
        "rim speed {rim} m/s vs centre speed {speed} m/s: not rolling"
    );
}

#[test]
fn bounce_heights_follow_each_material_restitution() {
    let r = Rules::V1;
    let drop_m = 1.0;
    let y_cm = (drop_m + r.ball_radius_m) * 100.0;
    let lv = level(vec![
        ball("steel", 100.0, y_cm, Material::Steel),
        ball("wood", 200.0, y_cm, Material::Wood),
        ball("rubber", 300.0, y_cm, Material::Rubber),
    ]);
    let mut s = sim(&lv, &scene(vec![]));
    let ids = ["steel", "wood", "rubber"].map(|n| part(&s, n));
    let mut bounced = [false; 3];
    let mut apex = [0.0f32; 3];
    for _ in 0..(2.0 / r.dt_s) as u32 {
        s.step();
        for (i, id) in ids.iter().enumerate() {
            let up = s.velocity(*id).unwrap()[1] > 0.0;
            bounced[i] |= up;
            if bounced[i] {
                apex[i] = apex[i].max(s.pose(*id).unwrap().y_m - r.ball_radius_m);
            }
        }
    }
    assert!(
        bounced.iter().all(|b| *b),
        "every ball should bounce at least once: {bounced:?}"
    );
    assert!(
        apex[2] > apex[1] && apex[1] > apex[0],
        "apex heights steel, wood, rubber: {apex:?}"
    );
    let expected = r.rubber.restitution * r.rubber.restitution * drop_m;
    assert!(
        close(apex[2], expected, 0.15),
        "rubber apex {} m, expected {expected} m",
        apex[2]
    );
}
