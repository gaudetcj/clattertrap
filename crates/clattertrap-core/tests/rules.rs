//! Rules version pin (CLAUDE.md invariant 5) and the shape of rules v1.

use clattertrap_core::rules::{RULES_VERSION, Rules};
use clattertrap_core::scene::Material;

#[test]
fn rules_version_is_1_until_the_first_gallery_entry() {
    assert_eq!(RULES_VERSION, 1);
    assert_eq!(Rules::V1.version, RULES_VERSION);
}

#[test]
fn v1_steps_at_120_hz_under_standard_gravity() {
    assert!((Rules::V1.dt_s * 120.0 - 1.0).abs() < 1e-6);
    assert!((Rules::V1.gravity_m_s2 - 9.81).abs() < 1e-6);
}

#[test]
fn materials_bounce_in_the_order_rubber_wood_steel_and_steel_is_densest() {
    let r = Rules::V1;
    let (steel, wood, rubber) = (
        r.material(Material::Steel),
        r.material(Material::Wood),
        r.material(Material::Rubber),
    );
    assert!(rubber.restitution > wood.restitution && wood.restitution > steel.restitution);
    assert!(steel.density > wood.density && steel.density > rubber.density);
}
