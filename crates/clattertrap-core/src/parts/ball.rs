//! Ball: a rolling disc of steel, wood, or rubber.

use super::{Look, m2};
use crate::events::PartId;
use crate::scene::Material;
use crate::world::Builder;

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], material: Material, b: &mut Builder<'_>) {
    let rules = *b.rules();
    let look = match material {
        Material::Steel => Look::Steel,
        Material::Wood => Look::Wood,
        Material::Rubber => Look::Rubber,
    };
    b.dynamic_ball(
        id,
        m2(centre_cm),
        rules.ball_radius_m,
        rules.material(material),
        look,
    );
}
