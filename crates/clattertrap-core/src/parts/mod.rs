//! One small module per part kind. Each builds its bodies, colliders, and joints from validated
//! parameters through `world::Builder` and names how it is drawn.

mod ball;
mod balloon;
mod basket;
mod beam;
mod bell;
mod bucket;
mod button;
mod crate_box;
mod domino;
mod funnel;
mod pendulum;
mod pin;
mod rope;
mod seesaw;
pub(crate) mod spring_pad;

use crate::events::PartId;
use crate::scene::PartSpec;
use crate::world::Builder;

/// Part kinds the simulator builds. Every kind of rules v1 is here.
pub const SUPPORTED: &[&str] = &[
    "beam",
    "funnel",
    "ball",
    "domino",
    "crate",
    "seesaw",
    "pendulum",
    "spring_pad",
    "bucket",
    "balloon",
    "pin",
    "rope",
    "bell",
    "button",
    "basket",
];

/// Surface finish the viewer draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// Tan plank or box.
    Wood,
    /// Grey metal.
    Steel,
    /// Dark rubber.
    Rubber,
    /// Cream tile with a dark edge.
    Domino,
    /// Red balloon.
    Balloon,
    /// Spring pad: a coil under a plate.
    Pad,
    /// Steel needle.
    Pin,
    /// Polished brass.
    Brass,
    /// Woven basket.
    Wicker,
}

/// What one body looks like, in metres, relative to its pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visual {
    /// A disc.
    Circle {
        /// Radius.
        r_m: f32,
        /// Finish.
        look: Look,
    },
    /// A rectangle before the body's rotation.
    Box {
        /// Full width.
        w_m: f32,
        /// Full height.
        h_m: f32,
        /// Finish.
        look: Look,
    },
    /// An open-top U whose origin is the floor's underside centre.
    Trough {
        /// Inner width.
        inner_w_m: f32,
        /// Inner height.
        inner_h_m: f32,
        /// Wall and floor thickness.
        wall_m: f32,
        /// Finish.
        look: Look,
    },
    /// A fixed pivot or pin point.
    Pivot,
}

/// Centimetres to metres.
#[must_use]
pub(crate) fn m(cm: f32) -> f32 {
    cm / 100.0
}

/// A centimetre point to metres.
#[must_use]
pub(crate) fn m2(cm: [f32; 2]) -> [f32; 2] {
    [m(cm[0]), m(cm[1])]
}

/// Degrees to radians.
#[must_use]
pub(crate) fn rad(deg: f32) -> f32 {
    deg * core::f32::consts::PI / 180.0
}

/// Adds a part to the world under construction. Ropes need both ends built first; `World::build`
/// adds them in a second pass.
pub(crate) fn build(spec: &PartSpec, id: PartId, b: &mut Builder<'_>) {
    match spec {
        PartSpec::Beam {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => beam::build(id, [*x_cm, *y_cm], *length_cm, *angle_deg, b),
        PartSpec::Funnel {
            x_cm,
            y_cm,
            mouth_cm,
            throat_cm,
            height_cm,
        } => {
            funnel::build(id, [*x_cm, *y_cm], [*mouth_cm, *throat_cm, *height_cm], b);
        }
        PartSpec::Ball {
            x_cm,
            y_cm,
            material,
        } => ball::build(id, [*x_cm, *y_cm], *material, b),
        PartSpec::Domino {
            x_cm,
            y_cm,
            angle_deg,
        } => domino::build(id, [*x_cm, *y_cm], *angle_deg, b),
        PartSpec::Crate {
            x_cm,
            y_cm,
            side_cm,
        } => crate_box::build(id, [*x_cm, *y_cm], *side_cm, b),
        PartSpec::Seesaw {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => seesaw::build(id, [*x_cm, *y_cm], *length_cm, *angle_deg, b),
        PartSpec::Pendulum {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => pendulum::build(id, [*x_cm, *y_cm], *length_cm, *angle_deg, b),
        PartSpec::SpringPad {
            x_cm,
            y_cm,
            angle_deg,
        } => spring_pad::build(id, [*x_cm, *y_cm], *angle_deg, b),
        PartSpec::Bucket {
            x_cm,
            y_cm,
            width_cm,
            height_cm,
        } => bucket::build(id, [*x_cm, *y_cm], [*width_cm, *height_cm], b),
        PartSpec::Balloon { x_cm, y_cm } => balloon::build(id, [*x_cm, *y_cm], b),
        PartSpec::Pin {
            x_cm,
            y_cm,
            angle_deg,
        } => pin::build(id, [*x_cm, *y_cm], *angle_deg, b),
        PartSpec::Rope {
            a,
            b: other,
            max_length_cm,
        } => rope::build(a, other, *max_length_cm, b),
        PartSpec::Bell {
            x_cm,
            y_cm,
            rope_cm,
        } => bell::build(id, [*x_cm, *y_cm], *rope_cm, b),
        PartSpec::Button {
            x_cm,
            y_cm,
            travel_cm,
        } => button::build(id, [*x_cm, *y_cm], *travel_cm, b),
        PartSpec::Basket {
            x_cm,
            y_cm,
            inner_width_cm,
            inner_height_cm,
        } => {
            basket::build(id, [*x_cm, *y_cm], [*inner_width_cm, *inner_height_cm], b);
        }
    }
}
