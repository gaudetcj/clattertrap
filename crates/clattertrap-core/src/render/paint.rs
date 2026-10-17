//! Flat colours per look and the outline of each visual, in the body's own metre frame.

use crate::parts::{Look, Visual};
use tiny_skia::{Color, Paint, Path, PathBuilder, Rect};

/// Radius of the dot that marks a pivot or a pin's anchor.
const PIVOT_R_M: f32 = 0.015;

/// The flat fill colour of a look, as the viewer paints it.
#[must_use]
pub fn look_rgb(look: Look) -> [u8; 3] {
    match look {
        Look::Wood => [0xc8, 0x95, 0x5a],
        Look::Steel => [0xb9, 0xc0, 0xc9],
        Look::Rubber => [0x2f, 0x2f, 0x33],
        Look::Domino => [0xf3, 0xea, 0xd7],
        Look::Balloon => [0xd9, 0x26, 0x26],
        Look::Pad => [0x6b, 0x74, 0x80],
        Look::Pin => [0xd8, 0xdd, 0xe3],
        Look::Brass => [0xe2, 0xb5, 0x3c],
        Look::Wicker => [0xc9, 0xa4, 0x6a],
    }
}

/// An opaque or translucent solid paint.
pub(super) fn solid(rgb: [u8; 3], alpha: f32) -> Paint<'static> {
    let mut paint = Paint::default();
    let a = (alpha * 255.0).clamp(0.0, 255.0);
    // alpha is 0..=1 here, so the rounding below stays in range.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    paint.set_color(Color::from_rgba8(
        rgb[0],
        rgb[1],
        rgb[2],
        libm::roundf(a) as u8,
    ));
    paint.anti_alias = true;
    paint
}

/// A darker shade of the same colour, for outlines.
pub(super) fn edge_of(rgb: [u8; 3]) -> [u8; 3] {
    rgb.map(|v| v / 2)
}

/// The visual's look, if it has one.
pub(super) fn look_of(visual: &Visual) -> Option<Look> {
    match visual {
        Visual::Circle { look, .. } | Visual::Box { look, .. } | Visual::Trough { look, .. } => {
            Some(*look)
        }
        Visual::Pivot => None,
    }
}

/// How far the visual reaches from its origin, for placing a label clear of it.
pub(super) fn reach_m(visual: &Visual) -> f32 {
    match visual {
        Visual::Circle { r_m, .. } => *r_m,
        Visual::Box { w_m, h_m, .. } => w_m.max(*h_m) / 2.0,
        Visual::Trough {
            inner_w_m, wall_m, ..
        } => inner_w_m / 2.0 + wall_m,
        Visual::Pivot => PIVOT_R_M,
    }
}

/// The outline of a visual around its origin, in metres with y up.
pub(super) fn outline(visual: &Visual) -> Option<Path> {
    match *visual {
        Visual::Circle { r_m, .. } => PathBuilder::from_circle(0.0, 0.0, r_m),
        Visual::Box { w_m, h_m, .. } => {
            Rect::from_xywh(-w_m / 2.0, -h_m / 2.0, w_m, h_m).map(PathBuilder::from_rect)
        }
        Visual::Trough {
            inner_w_m,
            inner_h_m,
            wall_m,
            ..
        } => trough(inner_w_m, inner_h_m, wall_m),
        Visual::Pivot => PathBuilder::from_circle(0.0, 0.0, PIVOT_R_M),
    }
}

/// A floor and two walls, the origin at the floor's underside centre.
fn trough(width_m: f32, depth_m: f32, wall_m: f32) -> Option<Path> {
    let full_w = width_m + 2.0 * wall_m;
    let height = depth_m + wall_m;
    let mut pb = PathBuilder::new();
    pb.push_rect(Rect::from_xywh(-full_w / 2.0, 0.0, full_w, wall_m)?);
    pb.push_rect(Rect::from_xywh(-full_w / 2.0, 0.0, wall_m, height)?);
    pb.push_rect(Rect::from_xywh(width_m / 2.0, 0.0, wall_m, height)?);
    pb.finish()
}
