//! Funnel: two fixed walls sloping inward from a wide mouth to a narrow throat.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::world::{Builder, Surface};

/// `size_cm` is mouth width, throat width, and height; `throat_cm` is the throat's centre.
pub(crate) fn build(id: PartId, throat_cm: [f32; 2], size_cm: [f32; 3], b: &mut Builder<'_>) {
    let rules = *b.rules();
    let [mouth, throat, height] = [m(size_cm[0]), m(size_cm[1]), m(size_cm[2])];
    let origin = m2(throat_cm);
    let surface = Surface {
        friction: rules.wood.friction,
        restitution: 0.0,
    };
    let dx = (mouth - throat) / 2.0;
    let length = libm::sqrtf(dx * dx + height * height);
    for side in [-1.0f32, 1.0] {
        // Each wall runs from the throat edge up and outward to the mouth edge.
        let centre = [
            origin[0] + side * (throat / 2.0 + dx / 2.0),
            origin[1] + height / 2.0,
        ];
        let angle = libm::atan2f(height, side * dx);
        b.fixed_box(
            id,
            centre,
            [length / 2.0, rules.wall_thickness_m / 2.0],
            angle,
            surface,
            Look::Wood,
        );
    }
}
