//! Spring pad: a fixed plate that gives whatever lands on it a fixed impulse along its up normal.

use super::{Look, m2, rad};
use crate::events::PartId;
use crate::world::{Builder, Surface};

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], angle_deg: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let half = [
        rules.spring_pad_size_m[0] / 2.0,
        rules.spring_pad_size_m[1] / 2.0,
    ];
    let surface = Surface {
        friction: rules.wood.friction,
        restitution: 0.0,
    };
    b.fixed_box(id, m2(centre_cm), half, rad(angle_deg), surface, Look::Pad);
}

/// Unit normal of the pad's launch face.
pub(crate) fn normal(angle_deg: f32) -> [f32; 2] {
    let a = rad(angle_deg);
    [-libm::sinf(a), libm::cosf(a)]
}
