//! The structural checks: ids, inventory, zones, and links.

use super::{FLOOR_NAME, Problem};
use crate::level::{Level, Zone};
use crate::parts;
use crate::scene::{PartSpec, Placed, Scene};

fn path(prefix: &str, index: usize, p: &Placed) -> String {
    format!("{prefix}[{index}] ({} {})", p.spec.kind(), p.id)
}

/// Both lists with their JSON path prefixes, fixed first.
pub(super) fn all<'a>(
    level: &'a Level,
    scene: &'a Scene,
) -> impl Iterator<Item = (String, &'a Placed)> {
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
