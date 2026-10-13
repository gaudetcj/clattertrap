//! Seesaw: a wooden plank hinged at its fixed centre.

use super::{Look, m, m2, rad};
use crate::events::PartId;
use crate::world::Builder;

pub(crate) fn build(
    id: PartId,
    centre_cm: [f32; 2],
    length_cm: f32,
    angle_deg: f32,
    b: &mut Builder<'_>,
) {
    let rules = *b.rules();
    let centre = m2(centre_cm);
    let half = [m(length_cm) / 2.0, rules.plank_thickness_m / 2.0];
    let plank = b.dynamic_box(id, centre, half, rad(angle_deg), rules.wood, Look::Wood);
    let pivot = b.fixed_anchor(id, centre);
    b.revolute(pivot, plank, [0.0, 0.0], [0.0, 0.0]);
}
