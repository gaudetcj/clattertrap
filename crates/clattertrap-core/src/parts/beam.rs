//! Beam: a fixed wooden plank.

use super::{Look, m, m2, rad};
use crate::events::PartId;
use crate::world::{Builder, Surface};

pub(crate) fn build(
    id: PartId,
    centre_cm: [f32; 2],
    length_cm: f32,
    angle_deg: f32,
    b: &mut Builder<'_>,
) {
    let rules = *b.rules();
    let half = [m(length_cm) / 2.0, rules.plank_thickness_m / 2.0];
    let surface = Surface {
        friction: rules.wood.friction,
        restitution: 0.0,
    };
    b.fixed_box(id, m2(centre_cm), half, rad(angle_deg), surface, Look::Wood);
}
