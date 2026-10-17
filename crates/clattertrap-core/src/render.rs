//! Rasterizes a scene at a step with tiny-skia: the world at a fixed scale, zones, a grid with
//! centimetre labels, every body in its look's flat colour, links as lines, and part ids beside parts.
//!
//! This is a reading aid for the `look` tool; the browser viewer is the art.

mod font;
mod paint;

pub use font::text_width;
pub use paint::look_rgb;

use crate::events::PartId;
use crate::level::Level;
use crate::parts::Visual;
use crate::scene::PartSpec;
use crate::sim::Simulator;
use crate::validate::ValidScene;
use crate::world::{BodyPose, Slot};
use tiny_skia::{Color, LineCap, Paint, PathBuilder, Pixmap, Rect, Stroke, StrokeDash, Transform};

const BACKGROUND: [u8; 3] = [0xf6, 0xf1, 0xe7];
const ZONE: [u8; 3] = [0x3b, 0x82, 0xf6];
const GRID: [u8; 3] = [0xd9, 0xd2, 0xc3];
const INK: [u8; 3] = [0x22, 0x22, 0x22];
const LINK: [u8; 3] = [0x33, 0x33, 0x33];
/// Grid spacing in centimetres.
const GRID_CM: f32 = 50.0;
/// Pixels per font dot for labels.
const TEXT_SCALE: f32 = 2.0;
/// Outline width in metres, so it scales with the frame.
const EDGE_M: f32 = 0.01;
/// Largest frame side, small enough that tiny-skia always accepts the pixmap.
const MAX_SIDE_PX: f32 = 8192.0;
/// Link line width in metres.
const LINK_M: f32 = 0.015;

/// PNG encoding failed.
#[derive(Debug, thiserror::Error)]
#[error("could not encode the PNG: {0}")]
pub struct PngError(String);

/// A level's picture frame: its size in pixels and how centimetres map to them.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    px_per_cm: f32,
    width_px: u32,
    height_px: u32,
    zones: Vec<[f32; 4]>,
}

impl Frame {
    /// A frame for `level` at `px_per_cm`, each side clamped to 1..=8192 pixels.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn new(level: &Level, px_per_cm: f32) -> Self {
        let px = |cm: f32| libm::roundf(cm * px_per_cm).clamp(1.0, MAX_SIDE_PX) as u32;
        Self {
            px_per_cm,
            width_px: px(level.world.width_cm),
            height_px: px(level.world.height_cm),
            zones: level
                .zones
                .iter()
                .map(|z| [z.x_cm, z.y_cm, z.width_cm, z.height_cm])
                .collect(),
        }
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width_px(&self) -> u32 {
        self.width_px
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height_px(&self) -> u32 {
        self.height_px
    }

    /// Paints the simulator's current state.
    ///
    /// # Panics
    /// Never: `new` clamps both sides to 1..=8192 pixels, which tiny-skia always accepts.
    #[must_use]
    pub fn draw(&self, sim: &Simulator, valid: &ValidScene) -> Pixmap {
        // Both sides are within 1..=8192 (`new` clamps them), so the pixmap exists.
        #[allow(clippy::expect_used)]
        let mut pixmap =
            Pixmap::new(self.width_px, self.height_px).expect("frame sides are within 1..=8192");
        pixmap.fill(colour(BACKGROUND));
        self.grid(&mut pixmap);
        self.zones(&mut pixmap);
        self.floor(&mut pixmap);
        let slots: Vec<Slot> = sim.slots().collect();
        self.links(&mut pixmap, &slots, valid);
        for slot in &slots {
            if let Some(pose) = slot.pose {
                self.body(&mut pixmap, &slot.visual, &pose);
            }
        }
        self.labels(&mut pixmap, &slots, sim.names(), valid.floor_id());
        pixmap
    }

    /// A centimetre point to pixels, y down.
    #[allow(clippy::cast_precision_loss)]
    fn px(&self, x_cm: f32, y_cm: f32) -> (f32, f32) {
        (
            x_cm * self.px_per_cm,
            self.height_px as f32 - y_cm * self.px_per_cm,
        )
    }

    /// Maps a body's metre frame (y up, rotated) onto pixels.
    #[allow(clippy::cast_precision_loss)]
    fn body_transform(&self, pose: &BodyPose) -> Transform {
        let k = self.px_per_cm * 100.0;
        let (s, c) = (libm::sinf(pose.angle_rad), libm::cosf(pose.angle_rad));
        Transform::from_row(
            k * c,
            -k * s,
            -k * s,
            -k * c,
            pose.x_m * k,
            self.height_px as f32 - pose.y_m * k,
        )
    }

    #[allow(clippy::cast_precision_loss)]
    fn grid(&self, pixmap: &mut Pixmap) {
        let thin = Stroke {
            width: 1.0,
            ..Stroke::default()
        };
        let line = paint::solid(GRID, 1.0);
        let ink = paint::solid(INK, 1.0);
        let (w, h) = (self.width_px as f32, self.height_px as f32);
        let mut x_cm = GRID_CM;
        while x_cm * self.px_per_cm < w {
            let (x, _) = self.px(x_cm, 0.0);
            stroke_line(pixmap, (x, 0.0), (x, h), &line, &thin);
            let label = format!("{x_cm}");
            let lx = x - text_width(&label, TEXT_SCALE) / 2.0;
            font::draw(pixmap, lx, h - 9.0 * TEXT_SCALE, &label, TEXT_SCALE, &ink);
            x_cm += GRID_CM;
        }
        let mut y_cm = GRID_CM;
        while y_cm * self.px_per_cm < h {
            let (_, y) = self.px(0.0, y_cm);
            stroke_line(pixmap, (0.0, y), (w, y), &line, &thin);
            font::draw(
                pixmap,
                3.0,
                y - 3.5 * TEXT_SCALE,
                &format!("{y_cm}"),
                TEXT_SCALE,
                &ink,
            );
            y_cm += GRID_CM;
        }
    }

    fn zones(&self, pixmap: &mut Pixmap) {
        let fill = paint::solid(ZONE, 0.08);
        let edge = paint::solid(ZONE, 0.6);
        let dashed = Stroke {
            width: 1.5,
            dash: StrokeDash::new(vec![6.0, 4.0], 0.0),
            ..Stroke::default()
        };
        for [x, y, w, h] in &self.zones {
            let (left, bottom) = self.px(*x, *y);
            let (right, top) = self.px(x + w, y + h);
            let Some(rect) = Rect::from_ltrb(left, top, right, bottom) else {
                continue;
            };
            pixmap.fill_rect(rect, &fill, Transform::identity(), None);
            let path = PathBuilder::from_rect(rect);
            pixmap.stroke_path(&path, &edge, &dashed, Transform::identity(), None);
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn floor(&self, pixmap: &mut Pixmap) {
        let (_, y) = self.px(0.0, 0.0);
        let stroke = Stroke {
            width: 2.0,
            ..Stroke::default()
        };
        let ink = paint::solid(INK, 1.0);
        stroke_line(
            pixmap,
            (0.0, y - 1.0),
            (self.width_px as f32, y - 1.0),
            &ink,
            &stroke,
        );
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
            font::draw(
                pixmap,
                x + reach_px + 4.0,
                y - 3.5 * TEXT_SCALE,
                name,
                TEXT_SCALE,
                &ink,
            );
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
