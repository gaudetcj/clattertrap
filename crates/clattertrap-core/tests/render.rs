//! The render module paints a scene at a step: world at a fixed scale, each look in its colour, links as
//! lines, ids lettered beside parts, and PNG bytes out.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

mod common;

use clattertrap_core::level::Level;
use clattertrap_core::parts::Look;
use clattertrap_core::render::{Frame, look_rgb, png, text, text_width};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::validate;
use common::{crate_box, level, pendulum, scene};
use tiny_skia::{Color, Pixmap};

const BELL_LEVEL: &str = include_str!("../../../levels/bell-drop/level.json");

fn rgb(p: &Pixmap, x: u32, y: u32) -> [u8; 3] {
    let c = p.pixel(x, y).unwrap();
    [c.red(), c.green(), c.blue()]
}

fn ink(c: [u8; 3]) -> bool {
    c.iter().all(|v| *v < 120)
}

fn bell_drop_at_start() -> (Frame, Pixmap) {
    let level: Level = serde_json::from_str(BELL_LEVEL).unwrap();
    let empty = Scene {
        level: level.id.clone(),
        parts: vec![],
    };
    let valid = validate(&level, &empty, &Rules::V1).unwrap();
    let sim = Simulator::new(&level, &valid, &Rules::V1);
    let frame = Frame::new(&level, 2.0);
    let pixmap = frame.draw(&sim, &valid);
    (frame, pixmap)
}

#[test]
fn the_frame_is_the_world_at_the_given_scale() {
    let (frame, pixmap) = bell_drop_at_start();
    assert_eq!((frame.width_px(), frame.height_px()), (800, 600));
    assert_eq!((pixmap.width(), pixmap.height()), (800, 600));
}

#[test]
fn parts_paint_their_look_at_their_centre_and_zones_tint_the_air() {
    let (_, p) = bell_drop_at_start();
    // The steel ball sits at (70, 280) cm and the brass bell at (360, 100) cm; y flips.
    assert_eq!(rgb(&p, 140, 40), look_rgb(Look::Steel));
    assert_eq!(rgb(&p, 720, 400), look_rgb(Look::Brass));
    let inside_zone = rgb(&p, 400, 300);
    assert_eq!(inside_zone, rgb(&p, 500, 200), "the zone is one flat tint");
    assert_ne!(
        inside_zone,
        rgb(&p, 20, 300),
        "outside the zone is a different tint"
    );
    assert!(!ink(inside_zone));
}

#[test]
fn a_part_id_is_lettered_beside_its_centre() {
    let (_, p) = bell_drop_at_start();
    // "ball0" starts right of the ball, whose radius is 12 px here; the outline ends by 14 px.
    let lettered = (156..230).any(|x| (28..52).any(|y| ink(rgb(&p, x, y))));
    assert!(lettered, "no ink right of the ball");
    let empty = (156..230).all(|x| (70..100).all(|y| !ink(rgb(&p, x, y))));
    assert!(empty, "ink below the label where nothing is drawn");
}

#[test]
fn text_sets_ink_in_a_five_by_seven_grid() {
    let mut p = Pixmap::new(20, 20).unwrap();
    p.fill(Color::WHITE);
    text(&mut p, 2.0, 2.0, "-", 1.0);
    assert!(
        ink(rgb(&p, 2, 5)) && ink(rgb(&p, 6, 5)),
        "the dash is row 3"
    );
    assert!(!ink(rgb(&p, 2, 2)) && !ink(rgb(&p, 2, 8)) && !ink(rgb(&p, 7, 5)));
    assert_eq!(text_width("ab", 1.0), 11.0, "5 px glyphs with 1 px between");
    assert_eq!(text_width("ab", 2.0), 22.0);
}

#[test]
fn a_pendulum_rod_joins_the_pin_to_the_weight() {
    let lv = level(vec![pendulum("p", 200.0, 200.0, 100.0, 0.0)]);
    let sc = scene(vec![]);
    let valid = validate(&lv, &sc, &Rules::V1).unwrap();
    let sim = Simulator::new(&lv, &valid, &Rules::V1);
    let p = Frame::new(&lv, 2.0).draw(&sim, &valid);
    // Pin at (200, 200), weight at (200, 100): the rod's midpoint is (200, 150) cm.
    assert!(
        ink(rgb(&p, 400, 300)),
        "no rod at the midpoint: {:?}",
        rgb(&p, 400, 300)
    );
    assert_eq!(
        rgb(&p, 400, 400),
        look_rgb(Look::Steel),
        "the weight hangs 1 m down"
    );
}

#[test]
fn png_bytes_start_with_the_signature() {
    let (_, p) = bell_drop_at_start();
    let bytes = png(&p).unwrap();
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
    );
    assert!(bytes.len() > 2000, "{}", bytes.len());
}

#[test]
fn a_label_near_the_edge_stays_inside_the_picture() {
    let lv = level(vec![crate_box("corner", 395.0, 6.0, 10.0)]);
    let sc = scene(vec![]);
    let valid = validate(&lv, &sc, &Rules::V1).unwrap();
    let sim = Simulator::new(&lv, &valid, &Rules::V1);
    let p = Frame::new(&lv, 2.0).draw(&sim, &valid);
    // "corner" is 70 px wide at this scale, so it is pulled left of the crate (x 780..800) and up off the edge.
    let lettered = (728..776).any(|x| (584..598).any(|y| ink(rgb(&p, x, y))));
    assert!(lettered, "no label left of the corner crate");
}
