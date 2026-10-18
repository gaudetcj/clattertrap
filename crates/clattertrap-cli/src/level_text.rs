//! A level as a contestant reads it: title, goal, description, fixed objects, zones, inventory, JSON.

use anyhow::Result;
use clattertrap_core::level::{Goal, Inventory, Level};
use clattertrap_core::scene::{Material, PartSpec};
use std::fmt::Write;

/// The goal's kind as it appears in JSON.
pub fn goal_kind(goal: &Goal) -> &'static str {
    match goal {
        Goal::Ring { .. } => "ring",
        Goal::Press { .. } => "press",
        Goal::Fill { .. } => "fill",
        Goal::Pop { .. } => "pop",
    }
}

/// The goal as a sentence fragment: `ring bell after touching post, then gate`.
pub fn goal_sentence(goal: &Goal) -> String {
    let mut s = match goal {
        Goal::Ring { target, .. } => format!("ring {target}"),
        Goal::Press { target, .. } => format!("press {target}"),
        Goal::Fill { target, count, .. } => {
            format!("bring {count} objects to rest inside {target}")
        }
        Goal::Pop { target, .. } => format!("pop {target}"),
    };
    if !goal.via().is_empty() {
        s.push_str(" after touching ");
        s.push_str(&goal.via().join(", then "));
    }
    s
}

fn material_name(m: Material) -> &'static str {
    match m {
        Material::Steel => "steel",
        Material::Wood => "wood",
        Material::Rubber => "rubber",
    }
}

/// One placed object in words.
pub fn describe(spec: &PartSpec) -> String {
    match *spec {
        PartSpec::Beam {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => {
            format!("beam {length_cm} cm long centred at ({x_cm}, {y_cm}), angle {angle_deg} deg")
        }
        PartSpec::Funnel {
            x_cm,
            y_cm,
            mouth_cm,
            throat_cm,
            height_cm,
        } => format!(
            "funnel with its throat at ({x_cm}, {y_cm}), mouth {mouth_cm} cm, throat {throat_cm} cm, {height_cm} cm tall"
        ),
        PartSpec::Ball {
            x_cm,
            y_cm,
            material,
        } => format!("{} ball at ({x_cm}, {y_cm})", material_name(material)),
        PartSpec::Domino {
            x_cm,
            y_cm,
            angle_deg,
        } => format!("domino at ({x_cm}, {y_cm}), angle {angle_deg} deg"),
        PartSpec::Crate {
            x_cm,
            y_cm,
            side_cm,
        } => {
            format!("crate {side_cm} cm square at ({x_cm}, {y_cm})")
        }
        PartSpec::Seesaw {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => format!(
            "seesaw {length_cm} cm long pivoting at ({x_cm}, {y_cm}), angle {angle_deg} deg"
        ),
        PartSpec::Pendulum {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        } => format!(
            "pendulum pinned at ({x_cm}, {y_cm}), rope {length_cm} cm, released at {angle_deg} deg"
        ),
        PartSpec::SpringPad {
            x_cm,
            y_cm,
            angle_deg,
        } => format!("spring pad at ({x_cm}, {y_cm}), angle {angle_deg} deg"),
        ref other => describe_rest(other),
    }
}

/// The containers, the balloon, the pin, the rope, and the goal objects.
fn describe_rest(spec: &PartSpec) -> String {
    match spec {
        PartSpec::Bucket {
            x_cm,
            y_cm,
            width_cm,
            height_cm,
        } => {
            format!("bucket {width_cm} by {height_cm} cm inside, floor centred at ({x_cm}, {y_cm})")
        }
        PartSpec::Balloon { x_cm, y_cm } => format!("balloon at ({x_cm}, {y_cm})"),
        PartSpec::Pin {
            x_cm,
            y_cm,
            angle_deg,
        } => format!("pin with its base at ({x_cm}, {y_cm}), pointing {angle_deg} deg"),
        PartSpec::Rope {
            a,
            b,
            max_length_cm,
        } => format!("rope from {a} to {b}, up to {max_length_cm} cm"),
        PartSpec::Bell {
            x_cm,
            y_cm,
            rope_cm,
        } => format!("bell at ({x_cm}, {y_cm}), hung {rope_cm} cm below its anchor"),
        PartSpec::Button {
            x_cm,
            y_cm,
            travel_cm,
        } => format!("button at ({x_cm}, {y_cm}) with {travel_cm} cm of travel"),
        PartSpec::Basket {
            x_cm,
            y_cm,
            inner_width_cm,
            inner_height_cm,
        } => format!(
            "basket {inner_width_cm} by {inner_height_cm} cm inside, floor centred at ({x_cm}, {y_cm})"
        ),
        other => describe(other),
    }
}

/// The inventory as `beam 4, seesaw 1`, skipping kinds with none.
pub fn inventory_text(inv: &Inventory) -> String {
    let counts = [
        ("beam", inv.beam),
        ("funnel", inv.funnel),
        ("ball", inv.ball),
        ("domino", inv.domino),
        ("crate", inv.crates),
        ("seesaw", inv.seesaw),
        ("pendulum", inv.pendulum),
        ("spring_pad", inv.spring_pad),
        ("bucket", inv.bucket),
        ("balloon", inv.balloon),
        ("pin", inv.pin),
        ("rope", inv.rope),
    ];
    let listed: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(kind, n)| format!("{kind} {n}"))
        .collect();
    if listed.is_empty() {
        "nothing".to_owned()
    } else {
        listed.join(", ")
    }
}

/// The `levels` listing: id, tier, goal kind, and title, one level per line.
#[must_use]
pub fn levels_text(levels: &[Level]) -> String {
    let mut text = String::new();
    let width = levels.iter().map(|l| l.id.len()).max().unwrap_or(0);
    for level in levels {
        // Writing to a String cannot fail.
        let _ = writeln!(
            text,
            "{:<width$} {}  {:<6} {}",
            level.id,
            level.tier,
            goal_kind(&level.goal),
            level.title
        );
    }
    text
}

/// The full `level` output.
///
/// # Errors
/// When the level cannot be serialized, which does not happen for a parsed level.
pub fn level_text(level: &Level) -> Result<String> {
    let mut lines = vec![
        format!("{}: {} (tier {})", level.id, level.title, level.tier),
        format!("Goal: {}.", goal_sentence(&level.goal)),
        format!(
            "Time limit: {} s. World: {} by {} cm.",
            level.time_limit_s, level.world.width_cm, level.world.height_cm
        ),
        String::new(),
        level.description.clone(),
        String::new(),
        "Fixed objects:".to_owned(),
    ];
    for p in &level.fixed {
        lines.push(format!("  {}: {}", p.id, describe(&p.spec)));
    }
    let zones: Vec<String> = level
        .zones
        .iter()
        .map(|z| {
            format!(
                "x {}..{}, y {}..{}",
                z.x_cm,
                z.x_cm + z.width_cm,
                z.y_cm,
                z.y_cm + z.height_cm
            )
        })
        .collect();
    lines.push(format!("Build zones: {}", zones.join("; ")));
    lines.push(format!("Inventory: {}", inventory_text(&level.inventory)));
    if level.showcase {
        lines.push("Showcase level: chain length is scored too.".to_owned());
    }
    lines.push(String::new());
    lines.push("Level JSON:".to_owned());
    lines.push(serde_json::to_string_pretty(level)?);
    lines.push(String::new());
    Ok(lines.join("\n"))
}
