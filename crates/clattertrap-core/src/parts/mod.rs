//! One small module per part kind. Each builds its bodies and colliders from validated parameters
//! through `world::Builder` and owns its drawing description.

mod ball;
mod beam;

use crate::events::PartId;
use crate::scene::PartSpec;
use crate::world::Builder;

/// Part kinds the simulator builds today. `validate` rejects the rest until their milestone step lands.
pub const SUPPORTED: &[&str] = &["beam", "ball"];

/// Surface finish the viewer draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// Tan plank.
    Wood,
    /// Grey metal.
    Steel,
    /// Dark rubber.
    Rubber,
}

/// What a part looks like, in metres, relative to its body pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visual {
    /// A disc.
    Circle {
        /// Radius.
        r_m: f32,
        /// Finish.
        look: Look,
    },
    /// An axis-aligned box before the body's rotation.
    Box {
        /// Full width.
        w_m: f32,
        /// Full height.
        h_m: f32,
        /// Finish.
        look: Look,
    },
}

/// Centimetres to metres.
#[must_use]
pub(crate) fn m(cm: f32) -> f32 {
    cm / 100.0
}

/// Degrees to radians.
#[must_use]
pub(crate) fn rad(deg: f32) -> f32 {
    deg * core::f32::consts::PI / 180.0
}

/// Adds a part to the world under construction.
pub(crate) fn build(spec: &PartSpec, id: PartId, b: &mut Builder<'_>) {
    match spec {
        PartSpec::Beam {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => beam::build(id, [*x_cm, *y_cm], *length_cm, *angle_deg, b),
        PartSpec::Ball {
            x_cm,
            y_cm,
            material,
        } => ball::build(id, [*x_cm, *y_cm], *material, b),
        // validate rejects every kind outside SUPPORTED before a world is built.
        other => unreachable!("validate admits only SUPPORTED kinds, got {}", other.kind()),
    }
}

/// The drawing description of a part.
#[must_use]
pub fn visual(spec: &PartSpec, rules: &crate::rules::Rules) -> Option<Visual> {
    match spec {
        PartSpec::Beam { length_cm, .. } => Some(beam::visual(*length_cm, rules)),
        PartSpec::Ball { material, .. } => Some(ball::visual(*material, rules)),
        _ => None,
    }
}
