//! Portal: two fixed ends a ball passes between. Each end is a sensor box that reports contacts
//! and stops nothing; the simulator moves a ball that enters one end out of the other, see
//! docs/physics.md#portal.

use super::{Look, m2, rad};
use crate::events::PartId;
use crate::rules::Rules;
use crate::scene::PartSpec;
use crate::world::Builder;

/// One end: where it is and which way it faces, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct End {
    pub centre_m: [f32; 2],
    /// Unit normal out of the face, into the room.
    pub normal: [f32; 2],
    /// The facing in radians, counterclockwise from +x.
    pub angle_rad: f32,
}

impl End {
    fn new(centre_cm: [f32; 2], angle_deg: f32) -> Self {
        let a = rad(angle_deg);
        Self {
            centre_m: m2(centre_cm),
            normal: [libm::cosf(a), libm::sinf(a)],
            angle_rad: a,
        }
    }
}

/// The orange end, then the blue end, of a portal spec; anything else has none.
pub(crate) fn ends(spec: &PartSpec) -> Option<[End; 2]> {
    match *spec {
        PartSpec::Portal {
            x_cm,
            y_cm,
            angle_deg,
            blue_x_cm,
            blue_y_cm,
            blue_angle_deg,
        } => Some([
            End::new([x_cm, y_cm], angle_deg),
            End::new([blue_x_cm, blue_y_cm], blue_angle_deg),
        ]),
        _ => None,
    }
}

pub(crate) fn build(id: PartId, spec: &PartSpec, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let half = [rules.portal_size_m[0] / 2.0, rules.portal_size_m[1] / 2.0];
    let Some([orange, blue]) = ends(spec) else {
        return;
    };
    for (end, look) in [(orange, Look::Orange), (blue, Look::Blue)] {
        // The box's local y is the face normal, so it turns a quarter back from the facing.
        let turn = end.angle_rad - core::f32::consts::FRAC_PI_2;
        b.sensor_box(id, end.centre_m, half, turn, look);
    }
}

/// Where a ball at `at_m` moving at `vel_m_s` that entered `entry` comes out of `exit`, and how it
/// moves: just clear of the exit face, at the same speed, turned as the two facings differ.
pub(crate) fn passage(
    entry: &End,
    exit: &End,
    vel_m_s: [f32; 2],
    ball_radius_m: f32,
    rules: &Rules,
) -> ([f32; 2], [f32; 2]) {
    let clear = rules.portal_size_m[1] / 2.0 + ball_radius_m + rules.portal_exit_gap_m;
    let at = [
        exit.centre_m[0] + exit.normal[0] * clear,
        exit.centre_m[1] + exit.normal[1] * clear,
    ];
    // Into the entry face maps onto out of the exit face: turn by the facings' difference plus a half turn.
    let turn = exit.angle_rad - entry.angle_rad + core::f32::consts::PI;
    let (s, c) = (libm::sinf(turn), libm::cosf(turn));
    let [vx, vy] = vel_m_s;
    (at, [c * vx - s * vy, s * vx + c * vy])
}

/// Which end a point is nearer, by index.
pub(crate) fn nearer(ends: &[End; 2], at_m: [f32; 2]) -> usize {
    let d2 = |e: &End| {
        let dx = at_m[0] - e.centre_m[0];
        let dy = at_m[1] - e.centre_m[1];
        dx * dx + dy * dy
    };
    usize::from(d2(&ends[1]) < d2(&ends[0]))
}
