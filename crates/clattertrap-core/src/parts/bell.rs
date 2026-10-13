//! Bell: a brass bell hanging from a bracket. It rings when a contact impulse exceeds the rules threshold.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::rules::MaterialProps;
use crate::world::Builder;

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], rope_cm: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let centre = m2(centre_cm);
    let rope = m(rope_cm);
    let props = MaterialProps {
        density: rules.bell_density,
        restitution: 0.3,
        friction: 0.2,
    };
    // The force threshold is the impulse threshold spread over one step, see docs/physics.md#bell.
    let threshold_n = rules.bell_min_impulse_ns / rules.dt_s;
    let bell = b.bell(
        id,
        centre,
        rules.bell_radius_m,
        props,
        threshold_n,
        Look::Brass,
    );
    let anchor = b.fixed_anchor(id, [centre[0], centre[1] + rope]);
    b.revolute(anchor, bell, [0.0, 0.0], [0.0, rope]);
}
