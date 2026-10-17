//! The structural checks: ids, inventory, zones, numeric ranges, and links.

use super::{FLOOR_NAME, Problem};
use crate::level::{Level, Zone};
use crate::parts;
use crate::rules::{Range, Rules};
use crate::scene::{PartSpec, Placed, Scene};

fn path(prefix: &str, index: usize, p: &Placed) -> String {
    format!("{prefix}[{index}] ({} {})", p.spec.kind(), p.id)
}

/// Both lists with their JSON path prefixes, fixed first.
fn all<'a>(level: &'a Level, scene: &'a Scene) -> impl Iterator<Item = (String, &'a Placed)> {
    let fixed = level
        .fixed
        .iter()
        .enumerate()
        .map(|(i, p)| (path("fixed", i, p), p));
    let parts = scene
        .parts
        .iter()
        .enumerate()
        .map(|(i, p)| (path("parts", i, p), p));
    fixed.chain(parts)
}

pub(super) fn level_header(level: &Level, scene: &Scene, out: &mut Vec<Problem>) {
    if scene.level != level.id {
        out.push(Problem::new(
            "level",
            format!(
                "is {:?} but this level is {:?}; set level to {:?}",
                scene.level, level.id, level.id
            ),
        ));
    }
    if !(1..=5).contains(&level.tier) {
        out.push(Problem::new(
            "level",
            format!(
                "tier {} is outside 1..5; set tier within that range",
                level.tier
            ),
        ));
    }
}

/// Checks ids and kinds and returns every id in order, valid or not.
pub(super) fn names(level: &Level, scene: &Scene, out: &mut Vec<Problem>) -> Vec<String> {
    let mut names: Vec<String> = Vec::with_capacity(level.fixed.len() + scene.parts.len() + 1);
    for (path, p) in all(level, scene) {
        if p.id == FLOOR_NAME {
            out.push(Problem::new(
                &path,
                format!("id {FLOOR_NAME:?} is reserved; rename the part"),
            ));
        } else if names.contains(&p.id) {
            out.push(Problem::new(
                &path,
                format!("id {:?} is already used; give this part a new id", p.id),
            ));
        }
        names.push(p.id.clone());
        if path.starts_with("parts") && p.spec.is_goal_object() {
            out.push(Problem::new(
                &path,
                format!(
                    "{} is a goal object only the level may place; remove it",
                    p.spec.kind()
                ),
            ));
            continue;
        }
        if !parts::SUPPORTED.contains(&p.spec.kind()) {
            out.push(Problem::new(
                &path,
                format!("{} is not simulated yet; use another kind", p.spec.kind()),
            ));
        }
    }
    if u16::try_from(names.len() + 1).is_err() {
        out.push(Problem::new(
            "parts",
            "too many parts; use fewer than 65535".to_owned(),
        ));
    }
    names
}

pub(super) fn inventory(level: &Level, scene: &Scene, out: &mut Vec<Problem>) {
    let mut seen: Vec<(&str, u8)> = Vec::new();
    for (i, p) in scene.parts.iter().enumerate() {
        if p.spec.is_goal_object() {
            continue;
        }
        let kind = p.spec.kind();
        let count = if let Some((_, n)) = seen.iter_mut().find(|(k, _)| *k == kind) {
            *n = n.saturating_add(1);
            *n
        } else {
            seen.push((kind, 1));
            1
        };
        let cap = level.inventory.allowed(kind);
        if count > cap {
            out.push(Problem::new(
                path("parts", i, p),
                format!(
                    "the inventory allows {cap} {kind} parts and this is number {count}; remove one"
                ),
            ));
        }
    }
}

fn describe(z: &Zone) -> String {
    format!(
        "x {}..{}, y {}..{}",
        z.x_cm,
        z.x_cm + z.width_cm,
        z.y_cm,
        z.y_cm + z.height_cm
    )
}

pub(super) fn zones(level: &Level, scene: &Scene, out: &mut Vec<Problem>) {
    let inside: Vec<String> = level.zones.iter().map(describe).collect();
    for (i, p) in scene.parts.iter().enumerate() {
        let Some(c) = p.spec.centre_cm() else {
            continue;
        };
        if !level.zones.iter().any(|z| z.contains_cm(c)) {
            out.push(Problem::new(
                path("parts", i, p),
                format!(
                    "centre ({}, {}) is outside every build zone; move it inside {}",
                    c[0],
                    c[1],
                    inside.join(" or ")
                ),
            ));
        }
    }
}

fn cm(v_m: f32) -> f32 {
    libm::roundf(v_m * 1000.0) / 10.0
}

fn check_range(path: &str, field: &str, value_cm: f32, range: Range, out: &mut Vec<Problem>) {
    let (lo, hi) = (cm(range.min_m), cm(range.max_m));
    if !value_cm.is_finite() || value_cm < lo || value_cm > hi {
        out.push(Problem::new(
            path,
            format!("{field} {value_cm} is outside {lo}..{hi}; set {field} within that range"),
        ));
    }
}

fn check_finite(path: &str, field: &str, value: f32, out: &mut Vec<Problem>) {
    if !value.is_finite() {
        out.push(Problem::new(
            path,
            format!("{field} is {value}; use a finite number"),
        ));
    }
}

pub(super) fn ranges(level: &Level, scene: &Scene, rules: &Rules, out: &mut Vec<Problem>) {
    for (path, p) in all(level, scene) {
        if let Some([x, y]) = p.spec.centre_cm() {
            check_finite(&path, "x_cm", x, out);
            check_finite(&path, "y_cm", y, out);
        }
        match &p.spec {
            PartSpec::Beam {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.beam_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Funnel {
                mouth_cm,
                throat_cm,
                height_cm,
                ..
            } => {
                check_range(&path, "mouth_cm", *mouth_cm, rules.funnel_mouth, out);
                let throat = Range {
                    min_m: rules.funnel_throat_min_m,
                    max_m: *mouth_cm / 100.0,
                };
                check_range(&path, "throat_cm", *throat_cm, throat, out);
                check_range(&path, "height_cm", *height_cm, rules.funnel_height, out);
            }
            PartSpec::Crate { side_cm, .. } => {
                check_range(&path, "side_cm", *side_cm, rules.crate_side, out);
            }
            PartSpec::Seesaw {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.seesaw_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Pendulum {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.pendulum_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Bucket {
                width_cm,
                height_cm,
                ..
            } => {
                check_range(&path, "width_cm", *width_cm, rules.bucket_size, out);
                check_range(&path, "height_cm", *height_cm, rules.bucket_size, out);
            }
            PartSpec::Rope { max_length_cm, .. } => check_range(
                &path,
                "max_length_cm",
                *max_length_cm,
                rules.rope_length,
                out,
            ),
            PartSpec::Bell { rope_cm, .. } => {
                check_range(&path, "rope_cm", *rope_cm, rules.bell_rope, out);
            }
            PartSpec::Button { travel_cm, .. } => {
                check_range(&path, "travel_cm", *travel_cm, rules.button_travel, out);
            }
            PartSpec::Basket {
                inner_width_cm,
                inner_height_cm,
                ..
            } => {
                check_range(
                    &path,
                    "inner_width_cm",
                    *inner_width_cm,
                    rules.bucket_size,
                    out,
                );
                check_range(
                    &path,
                    "inner_height_cm",
                    *inner_height_cm,
                    rules.bucket_size,
                    out,
                );
            }
            PartSpec::Domino { angle_deg, .. }
            | PartSpec::SpringPad { angle_deg, .. }
            | PartSpec::Pin { angle_deg, .. } => {
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Ball { .. } | PartSpec::Balloon { .. } => {}
        }
    }
}

fn link_problem(
    end: &str,
    target: &str,
    level: &Level,
    scene: &Scene,
    names: &[String],
) -> Option<String> {
    if !names.iter().any(|n| n == target) {
        return Some(format!(
            "{end} = {target:?} names no part; link two placed parts"
        ));
    }
    let is_rope =
        all(level, scene).any(|(_, p)| p.id == target && matches!(p.spec, PartSpec::Rope { .. }));
    is_rope.then(|| format!("{end} = {target:?} is a rope; link {end} to a solid part"))
}

pub(super) fn links(level: &Level, scene: &Scene, names: &[String], out: &mut Vec<Problem>) {
    for (path, p) in all(level, scene) {
        if let PartSpec::Rope { a, b, .. } = &p.spec {
            for (end, target) in [("a", a), ("b", b)] {
                if let Some(message) = link_problem(end, target, level, scene, names) {
                    out.push(Problem::new(&path, message));
                }
            }
        }
    }
    for (i, id) in level.goal.via().iter().enumerate() {
        if !names.iter().any(|n| n == id) {
            out.push(Problem::new(
                format!("goal.via[{i}]"),
                format!("{id:?} names no part; name a fixed object or a placed part"),
            ));
        }
    }
}

/// The goal names a fixed object whose kind matches the goal.
pub(super) fn goal_target(level: &Level, out: &mut Vec<Problem>) {
    let wanted = match level.goal {
        crate::level::Goal::Ring { .. } => "bell",
        crate::level::Goal::Press { .. } => "button",
        crate::level::Goal::Fill { .. } => "basket",
        crate::level::Goal::Pop { .. } => "balloon",
    };
    let target = level.goal.target();
    let found = level
        .fixed
        .iter()
        .any(|p| p.id == target && p.spec.kind() == wanted);
    if !found {
        out.push(Problem::new(
            "goal",
            format!("target {target:?} names no fixed {wanted}; add a {wanted} with that id to fixed or change the target"),
        ));
    }
}
