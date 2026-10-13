//! Domino: a standing wooden tile that tips when pushed.

use super::{Look, m2, rad};
use crate::events::PartId;
use crate::world::Builder;

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], angle_deg: f32, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let half = [rules.domino_size_m[0] / 2.0, rules.domino_size_m[1] / 2.0];
    b.dynamic_box(
        id,
        m2(centre_cm),
        half,
        rad(angle_deg),
        rules.wood,
        Look::Domino,
    );
}
