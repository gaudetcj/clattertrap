//! The paper behind the parts: the label margins, the centimetre grid, the build zones, and the floor.

use super::{
    Frame, GRID_CM, INK, MARGIN_BOTTOM_PX, MARGIN_LEFT_PX, TEXT_SCALE, font, paint, stroke_line,
    text_width,
};
use tiny_skia::{PathBuilder, Pixmap, Rect, Stroke, StrokeDash, Transform};

const MARGIN: [u8; 3] = [0xe9, 0xe2, 0xd3];
const ZONE: [u8; 3] = [0x3b, 0x82, 0xf6];
const GRID: [u8; 3] = [0xd9, 0xd2, 0xc3];

impl Frame {
    /// Everything drawn before the bodies.
    pub(super) fn paper(&self, pixmap: &mut Pixmap) {
        self.margins(pixmap);
        self.grid(pixmap);
        self.zones(pixmap);
        self.floor(pixmap);
    }

    /// A left strip for y labels and a bottom strip for x labels, in a darker paper tone.
    fn margins(&self, pixmap: &mut Pixmap) {
        let paint = paint::solid(MARGIN, 1.0);
        let (w, h) = (self.width_px(), self.height_px());
        let strips = [
            Rect::from_xywh(0.0, 0.0, MARGIN_LEFT_PX, h),
            Rect::from_xywh(0.0, self.world_h_px, w, MARGIN_BOTTOM_PX),
        ];
        for rect in strips.into_iter().flatten() {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    /// Lines every `GRID_CM`, with the centimetre value lettered in the margin.
    fn grid(&self, pixmap: &mut Pixmap) {
        let thin = Stroke {
            width: 1.0,
            ..Stroke::default()
        };
        let line = paint::solid(GRID, 1.0);
        let ink = paint::solid(INK, 1.0);
        let mut x_cm = GRID_CM;
        while x_cm * self.px_per_cm < self.world_w_px {
            let (x, _) = self.px(x_cm, 0.0);
            stroke_line(pixmap, (x, 0.0), (x, self.world_h_px), &line, &thin);
            let label = format!("{x_cm}");
            let lx = x - text_width(&label, TEXT_SCALE) / 2.0;
            font::draw(pixmap, lx, self.world_h_px + 3.0, &label, TEXT_SCALE, &ink);
            x_cm += GRID_CM;
        }
        let mut y_cm = GRID_CM;
        while y_cm * self.px_per_cm < self.world_h_px {
            let (_, y) = self.px(0.0, y_cm);
            stroke_line(
                pixmap,
                (MARGIN_LEFT_PX, y),
                (self.width_px(), y),
                &line,
                &thin,
            );
            let label = format!("{y_cm}");
            let lx = MARGIN_LEFT_PX - 4.0 - text_width(&label, TEXT_SCALE);
            font::draw(pixmap, lx, y - 3.5 * TEXT_SCALE, &label, TEXT_SCALE, &ink);
            y_cm += GRID_CM;
        }
    }

    /// Build zones as a light tint with a dashed edge.
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

    /// The floor line along the bottom of the world.
    fn floor(&self, pixmap: &mut Pixmap) {
        let (_, y) = self.px(0.0, 0.0);
        let stroke = Stroke {
            width: 2.0,
            ..Stroke::default()
        };
        let ink = paint::solid(INK, 1.0);
        let from = (MARGIN_LEFT_PX, y - 1.0);
        let to = (self.width_px(), y - 1.0);
        stroke_line(pixmap, from, to, &ink, &stroke);
    }
}
