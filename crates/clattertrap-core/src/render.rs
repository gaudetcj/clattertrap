//! Rasterizes a scene at a step with tiny-skia: the world at a fixed scale beside two label margins,
//! zones, a grid with centimetre labels, every body in its look's flat colour, links as lines, and
//! part ids beside parts.
//!
//! This is a reading aid for the `look` tool; the browser viewer is the art.

mod font;
mod paint;
mod paper;

pub use font::text_width;
pub use paint::look_rgb;

use crate::events::PartId;
use crate::level::Level;
use crate::parts::Visual;
use crate::scene::PartSpec;
use crate::sim::Simulator;
use crate::validate::ValidScene;
use crate::world::{BodyPose, Slot};
use tiny_skia::{Color, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const BACKGROUND: [u8; 3] = [0xf6, 0xf1, 0xe7];
const INK: [u8; 3] = [0x22, 0x22, 0x22];
const LINK: [u8; 3] = [0x33, 0x33, 0x33];
/// Grid spacing in centimetres.
const GRID_CM: f32 = 50.0;
/// Pixels per font dot for labels.
const TEXT_SCALE: f32 = 2.0;
/// Outline width in metres, so it scales with the frame.
const EDGE_M: f32 = 0.01;
/// Link line width in metres.
const LINK_M: f32 = 0.015;
/// Strip left of the world for the y labels, in pixels.
const MARGIN_LEFT_PX: f32 = 40.0;
/// Strip below the world for the x labels, in pixels.
const MARGIN_BOTTOM_PX: f32 = 20.0;
/// Largest side of the world area, small enough that tiny-skia always accepts the pixmap.
const MAX_WORLD_SIDE_PX: f32 = 8000.0;

/// PNG encoding failed.
#[derive(Debug, thiserror::Error)]
#[error("could not encode the PNG: {0}")]
pub struct PngError(String);

/// A level's picture frame: the world area in pixels, the margins around it, and the centimetre scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    px_per_cm: f32,
    /// Width of the world area, right of the left margin.
    world_w_px: f32,
    /// Height of the world area, above the bottom margin.
    world_h_px: f32,
    zones: Vec<[f32; 4]>,
}

impl Frame {
    /// A frame for `level` at `px_per_cm`; the world area is clamped to 1..=8000 pixels a side.
    #[must_use]
    pub fn new(level: &Level, px_per_cm: f32) -> Self {
        let px = |cm: f32| libm::roundf(cm * px_per_cm).clamp(1.0, MAX_WORLD_SIDE_PX);
        Self {
            px_per_cm,
            world_w_px: px(level.world.width_cm),
            world_h_px: px(level.world.height_cm),
            zones: level
                .zones
                .iter()
                .map(|z| [z.x_cm, z.y_cm, z.width_cm, z.height_cm])
                .collect(),
        }
    }

    /// Width in pixels, margin included.
    #[must_use]
    pub fn width_px(&self) -> f32 {
        self.world_w_px + MARGIN_LEFT_PX
    }

    /// Height in pixels, margin included.
    #[must_use]
    pub fn height_px(&self) -> f32 {
        self.world_h_px + MARGIN_BOTTOM_PX
    }

    /// Paints the simulator's current state.
    ///
    /// # Panics
    /// Never: `new` clamps the world area to 1..=8000 pixels a side, which tiny-skia always accepts.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn draw(&self, sim: &Simulator, valid: &ValidScene) -> Pixmap {
        // Both sides are whole numbers within 41..=8040 (`new` clamps them), so the pixmap exists.
        #[allow(clippy::expect_used)]
        let mut pixmap = Pixmap::new(self.width_px() as u32, self.height_px() as u32)
            .expect("frame sides are within 41..=8040");
        pixmap.fill(colour(BACKGROUND));
        self.paper(&mut pixmap);
        let slots: Vec<Slot> = sim.slots().collect();
        self.links(&mut pixmap, &slots, valid);
        // The floor slab would cover the bottom margin; the floor line stands for it.
        let floor = valid.floor_id();
        for slot in slots.iter().filter(|s| s.part != floor) {
            if let Some(pose) = slot.pose {
                self.body(&mut pixmap, &slot.visual, &pose);
            }
        }
        self.labels(&mut pixmap, &slots, sim.names(), floor);
        pixmap
    }

    /// A centimetre point to pixels, y down.
    fn px(&self, x_cm: f32, y_cm: f32) -> (f32, f32) {
        (
            MARGIN_LEFT_PX + x_cm * self.px_per_cm,
            self.world_h_px - y_cm * self.px_per_cm,
        )
    }

    /// Maps a body's metre frame (y up, rotated) onto pixels.
    fn body_transform(&self, pose: &BodyPose) -> Transform {
        let k = self.px_per_cm * 100.0;
        let (s, c) = (libm::sinf(pose.angle_rad), libm::cosf(pose.angle_rad));
        let (tx, ty) = self.px(pose.x_m * 100.0, pose.y_m * 100.0);
        Transform::from_row(k * c, -k * s, -k * s, -k * c, tx, ty)
    }

    /// Pivot to body lines for pendulums, bells, and seesaws, and rope lines between parts.
    fn links(&self, pixmap: &mut Pixmap, slots: &[Slot], valid: &ValidScene) {
        let stroke = Stroke {
            width: LINK_M * 100.0 * self.px_per_cm,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        let ink = paint::solid(LINK, 1.0);
        let mut pairs: Vec<(BodyPose, BodyPose)> = Vec::new();
        for slot in slots {
            if let (Visual::Pivot, Some(pivot)) = (slot.visual, slot.pose)
                && let Some(body) = first_body(slots, slot.part)
            {
                pairs.push((pivot, body));
            }
        }
        for placed in valid.placed() {
            if let PartSpec::Rope { a, b, .. } = &placed.spec
                && let (Some(a), Some(b)) = (valid.part_id(a), valid.part_id(b))
                && let (Some(a), Some(b)) = (first_body(slots, a), first_body(slots, b))
            {
                pairs.push((a, b));
            }
        }
        for (from, to) in pairs {
            let from = self.px(from.x_m * 100.0, from.y_m * 100.0);
            let to = self.px(to.x_m * 100.0, to.y_m * 100.0);
            stroke_line(pixmap, from, to, &ink, &stroke);
        }
    }

    fn body(&self, pixmap: &mut Pixmap, visual: &Visual, pose: &BodyPose) {
        let Some(path) = paint::outline(visual) else {
            return;
        };
        let rgb = paint::look_of(visual).map_or(INK, look_rgb);
        let transform = self.body_transform(pose);
        let fill = paint::solid(rgb, 1.0);
        pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, transform, None);
        let edge = paint::solid(paint::edge_of(rgb), 1.0);
        let stroke = Stroke {
            width: EDGE_M,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, &edge, &stroke, transform, None);
    }

    /// A label origin pulled inside the world area, with a 2 px margin.
    fn clamp_label(&self, x: f32, y: f32, name: &str) -> (f32, f32) {
        let right = self.width_px() - text_width(name, TEXT_SCALE) - 2.0;
        let bottom = self.world_h_px - 7.0 * TEXT_SCALE - 2.0;
        (
            x.min(right).max(MARGIN_LEFT_PX + 2.0),
            y.min(bottom).max(2.0),
        )
    }

    /// Each part's id just right of its first body, clear of the outline.
    fn labels(&self, pixmap: &mut Pixmap, slots: &[Slot], names: &[String], floor: PartId) {
        let ink = paint::solid(INK, 1.0);
        for (i, name) in names.iter().enumerate() {
            let Ok(part) = u16::try_from(i).map(PartId) else {
                continue;
            };
            if part == floor {
                continue;
            }
            let Some(slot) = slots
                .iter()
                .find(|s| s.part == part && s.visual != Visual::Pivot && s.pose.is_some())
            else {
                continue;
            };
            let Some(pose) = slot.pose else { continue };
            let reach_px = paint::reach_m(&slot.visual) * 100.0 * self.px_per_cm;
            let (x, y) = self.px(pose.x_m * 100.0, pose.y_m * 100.0);
            let (x, y) = self.clamp_label(x + reach_px + 4.0, y - 3.5 * TEXT_SCALE, name);
            font::draw(pixmap, x, y, name, TEXT_SCALE, &ink);
        }
    }
}

/// The first body of a part that is not a pivot and still exists.
fn first_body(slots: &[Slot], part: PartId) -> Option<BodyPose> {
    slots
        .iter()
        .find(|s| s.part == part && s.visual != Visual::Pivot)
        .and_then(|s| s.pose)
}

fn colour(rgb: [u8; 3]) -> Color {
    Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255)
}

fn stroke_line(
    pixmap: &mut Pixmap,
    from: (f32, f32),
    to: (f32, f32),
    paint: &Paint,
    stroke: &Stroke,
) {
    let mut pb = PathBuilder::new();
    pb.move_to(from.0, from.1);
    pb.line_to(to.0, to.1);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
    }
}

/// Letters `s` in ink with its top-left at (`x`, `y`), `scale` pixels per dot.
pub fn text(pixmap: &mut Pixmap, x: f32, y: f32, s: &str, scale: f32) {
    font::draw(pixmap, x, y, s, scale, &paint::solid(INK, 1.0));
}

/// Encodes a pixmap as PNG bytes.
///
/// # Errors
/// When the encoder fails, which an in-memory target does not do in practice.
pub fn png(pixmap: &Pixmap) -> Result<Vec<u8>, PngError> {
    pixmap.encode_png().map_err(|e| PngError(e.to_string()))
}
