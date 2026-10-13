//! Pendulum: a steel weight on a rigid rope from a fixed pin.

use super::{Look, m, m2, rad};
use crate::events::PartId;
use crate::world::Builder;

pub(crate) fn build(
    id: PartId,
    pin_cm: [f32; 2],
    length_cm: f32,
    angle_deg: f32,
    b: &mut Builder<'_>,
) {
    let rules = *b.rules();
    let pin = m2(pin_cm);
    let (len, a) = (m(length_cm), rad(angle_deg));
    // Angle is measured from straight down, counterclockwise positive, see docs/physics.md#pendulum.
    let weight_at = [pin[0] + len * libm::sinf(a), pin[1] - len * libm::cosf(a)];
    let weight = b.dynamic_ball(
        id,
        weight_at,
        rules.pendulum_radius_m,
        rules.steel,
        Look::Steel,
    );
    let anchor = b.fixed_anchor(id, pin);
    b.revolute(
        anchor,
        weight,
        [0.0, 0.0],
        [pin[0] - weight_at[0], pin[1] - weight_at[1]],
    );
}
