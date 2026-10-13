//! Crate: a wooden box.

use super::{Look, m, m2};
use crate::events::PartId;
use crate::world::Builder;

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], side_cm: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let half = m(side_cm) / 2.0;
    b.dynamic_box(id, m2(centre_cm), [half, half], 0.0, rules.wood, Look::Wood);
}
