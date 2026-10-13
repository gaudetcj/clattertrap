//! Basket: a fixed open-top container. Filled when enough objects rest inside.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::world::{Builder, Trough};

/// `origin_cm` is the floor's underside centre; `inner_cm` is the inner width and height.
pub(crate) fn build(id: PartId, origin_cm: [f32; 2], inner_cm: [f32; 2], b: &mut Builder<'_>) {
    let rules = *b.rules();
    let shape = Trough {
        inner_m: [m(inner_cm[0]), m(inner_cm[1])],
        wall_m: rules.basket_wall_m,
    };
    b.trough(id, false, m2(origin_cm), shape, rules.wood, Look::Wicker);
}
