//! Bucket: an open-top wooden container that moves.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::world::{Builder, Trough};

/// `inner_cm` is the inner width and height; `origin_cm` is the floor's underside centre.
pub(crate) fn build(id: PartId, origin_cm: [f32; 2], inner_cm: [f32; 2], b: &mut Builder<'_>) {
    let rules = *b.rules();
    let shape = Trough {
        inner_m: [m(inner_cm[0]), m(inner_cm[1])],
        wall_m: rules.wall_thickness_m,
    };
    b.trough(id, true, m2(origin_cm), shape, rules.wood, Look::Wood);
}
