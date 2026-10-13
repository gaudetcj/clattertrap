//! Balloon: a light disc that rises until a pin pops it.

use super::{Look, m2};
use crate::events::PartId;
use crate::world::{Builder, Lift};

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], b: &mut Builder<'_>) {
    let rules = *b.rules();
    let lift = Lift {
        density: rules.balloon_density,
        gravity_scale: rules.balloon_gravity_scale,
        damping: rules.balloon_damping,
    };
    b.balloon(
        id,
        m2(centre_cm),
        rules.balloon_radius_m,
        lift,
        Look::Balloon,
    );
}
