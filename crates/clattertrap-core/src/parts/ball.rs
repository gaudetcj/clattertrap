//! Ball: a rolling disc of steel, wood, or rubber.

use super::{Look, Visual, m};
use crate::events::PartId;
use crate::rules::Rules;
use crate::scene::Material;
use crate::world::Builder;

pub(crate) fn build(id: PartId, centre_cm: [f32; 2], material: Material, b: &mut Builder<'_>) {
    let rules = *b.rules();
    b.dynamic_ball(
        id,
        [m(centre_cm[0]), m(centre_cm[1])],
        rules.ball_radius_m,
        rules.material(material),
    );
}

pub(crate) fn visual(material: Material, rules: &Rules) -> Visual {
    let look = match material {
        Material::Steel => Look::Steel,
        Material::Wood => Look::Wood,
        Material::Rubber => Look::Rubber,
    };
    Visual::Circle {
        r_m: rules.ball_radius_m,
        look,
    }
}
