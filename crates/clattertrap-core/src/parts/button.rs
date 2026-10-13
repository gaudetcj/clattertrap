//! Button: a sprung cap on a fixed base. Pressed when the cap travels the level's `travel_cm`.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::world::{Builder, Slide, Surface};

/// Thickness of the fixed base plate under the cap.
pub(crate) const BASE_M: f32 = 0.02;

/// `at_cm` is the centre of the cap's resting underside.
pub(crate) fn build(id: PartId, at_cm: [f32; 2], travel_cm: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let at = m2(at_cm);
    let travel = m(travel_cm);
    let [cap_w, cap_h] = rules.button_cap_size_m;
    // The cap goes first so the part's first body is the one whose travel is measured.
    let cap = b.dynamic_box(
        id,
        [at[0], at[1] + cap_h / 2.0],
        [cap_w / 2.0, cap_h / 2.0],
        0.0,
        rules.wood,
        Look::Steel,
    );
    let base_at = [at[0], at[1] - travel - BASE_M / 2.0];
    let surface = Surface {
        friction: rules.wood.friction,
        restitution: 0.0,
    };
    let base = b.fixed_box(
        id,
        base_at,
        [cap_w / 2.0 + 0.02, BASE_M / 2.0],
        0.0,
        surface,
        Look::Steel,
    );
    let slide = Slide {
        axis: [0.0, 1.0],
        limits_m: [-travel, 0.0],
        stiffness_n_m: rules.button_spring_n_m,
        damping: rules.button_damping,
    };
    b.prismatic(
        base,
        cap,
        [[0.0, travel + BASE_M / 2.0], [0.0, -cap_h / 2.0]],
        slide,
    );
}
