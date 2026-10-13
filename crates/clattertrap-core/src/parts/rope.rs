//! Rope: links two parts so they stay within a maximum distance. It has no body of its own.

use super::m;
use crate::world::Builder;

pub(crate) fn build(a: &str, other: &str, max_length_cm: f32, b: &mut Builder<'_>) {
    // validate guarantees both ends name parts that are not ropes, so both lookups succeed.
    let ends = b.part_id(a).zip(b.part_id(other));
    if let Some((ha, hb)) = ends.and_then(|(pa, pb)| b.first_body(pa).zip(b.first_body(pb))) {
        b.rope(ha, hb, m(max_length_cm));
    }
}
