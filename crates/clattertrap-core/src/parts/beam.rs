//! Beam: a fixed wooden plank.

use super::{Look, Visual, m, rad};
use crate::events::PartId;
use crate::rules::Rules;
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
    b.fixed_box(
        id,
        [m(centre_cm[0]), m(centre_cm[1])],
        half,
        rad(angle_deg),
        surface,
    );
}

pub(crate) fn visual(length_cm: f32, rules: &Rules) -> Visual {
    Visual::Box {
        w_m: m(length_cm),
        h_m: rules.plank_thickness_m,
        look: Look::Wood,
    }
}
