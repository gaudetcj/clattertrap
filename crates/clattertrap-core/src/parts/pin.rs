//! Pin: a fixed needle that pops balloons. Its base sits at the given point and it points along `angle_deg`.

use super::{Look, m2, rad};
use crate::events::PartId;
use crate::world::{Builder, Surface};

pub(crate) fn build(id: PartId, base_cm: [f32; 2], angle_deg: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let a = rad(angle_deg);
    let half_len = rules.pin_size_m[1] / 2.0;
    let base = m2(base_cm);
    let centre = [
        base[0] + half_len * libm::cosf(a),
        base[1] + half_len * libm::sinf(a),
    ];
    let surface = Surface {
        friction: rules.steel.friction,
        restitution: 0.0,
    };
    b.fixed_box(
        id,
        centre,
        [half_len, rules.pin_size_m[0] / 2.0],
        a,
        surface,
        Look::Pin,
    );
}
