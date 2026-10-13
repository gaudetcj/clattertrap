//! Shared builders for physics tests: a 4 by 3 m level with everything allowed, and a validated simulator.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use clattertrap_core::events::PartId;
use clattertrap_core::level::{Goal, Inventory, Level, WorldSize, Zone};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::{Material, PartSpec, Placed, Scene};
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::validate;

pub const PI: f32 = std::f32::consts::PI;

/// A level whose goal is a bell tucked in the top-right corner, out of the way of physics tests.
pub fn level(mut fixed: Vec<Placed>) -> Level {
    fixed.push(placed(
        "bell",
        PartSpec::Bell {
            x_cm: 390.0,
            y_cm: 280.0,
            rope_cm: 10.0,
        },
    ));
    level_with_goal(
        fixed,
        Goal::Ring {
            target: "bell".to_owned(),
            via: vec![],
        },
    )
}

pub fn level_with_goal(fixed: Vec<Placed>, goal: Goal) -> Level {
    Level {
        id: "test".to_owned(),
        title: "test".to_owned(),
        rules_version: 1,
        world: WorldSize {
            width_cm: 400.0,
            height_cm: 300.0,
        },
        time_limit_s: 20.0,
        fixed,
        zones: vec![Zone {
            x_cm: 0.0,
            y_cm: 0.0,
            width_cm: 400.0,
            height_cm: 300.0,
        }],
        inventory: Inventory {
            beam: 8,
            funnel: 2,
            ball: 8,
            domino: 16,
            crates: 4,
            seesaw: 2,
            pendulum: 2,
            spring_pad: 2,
            bucket: 2,
            balloon: 2,
            pin: 2,
            rope: 2,
        },
        goal,
        showcase: false,
    }
}

pub fn scene(parts: Vec<Placed>) -> Scene {
    Scene {
        level: "test".to_owned(),
        parts,
    }
}

pub fn placed(id: &str, spec: PartSpec) -> Placed {
    Placed {
        id: id.to_owned(),
        spec,
    }
}

pub fn ball(id: &str, x_cm: f32, y_cm: f32, material: Material) -> Placed {
    placed(
        id,
        PartSpec::Ball {
            x_cm,
            y_cm,
            material,
        },
    )
}

pub fn beam(id: &str, x_cm: f32, y_cm: f32, length_cm: f32, angle_deg: f32) -> Placed {
    placed(
        id,
        PartSpec::Beam {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        },
    )
}

pub fn domino(id: &str, x_cm: f32, y_cm: f32) -> Placed {
    placed(
        id,
        PartSpec::Domino {
            x_cm,
            y_cm,
            angle_deg: 0.0,
        },
    )
}

pub fn crate_box(id: &str, x_cm: f32, y_cm: f32, side_cm: f32) -> Placed {
    placed(
        id,
        PartSpec::Crate {
            x_cm,
            y_cm,
            side_cm,
        },
    )
}

pub fn seesaw(id: &str, x_cm: f32, y_cm: f32, length_cm: f32, angle_deg: f32) -> Placed {
    placed(
        id,
        PartSpec::Seesaw {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        },
    )
}

pub fn pendulum(id: &str, x_cm: f32, y_cm: f32, length_cm: f32, angle_deg: f32) -> Placed {
    placed(
        id,
        PartSpec::Pendulum {
            x_cm,
            y_cm,
            length_cm,
            angle_deg,
        },
    )
}

pub fn rope(id: &str, a: &str, b: &str, max_length_cm: f32) -> Placed {
    placed(
        id,
        PartSpec::Rope {
            a: a.to_owned(),
            b: b.to_owned(),
            max_length_cm,
        },
    )
}

pub fn spring_pad(id: &str, x_cm: f32, y_cm: f32, angle_deg: f32) -> Placed {
    placed(
        id,
        PartSpec::SpringPad {
            x_cm,
            y_cm,
            angle_deg,
        },
    )
}

pub fn balloon(id: &str, x_cm: f32, y_cm: f32) -> Placed {
    placed(id, PartSpec::Balloon { x_cm, y_cm })
}

pub fn pin(id: &str, x_cm: f32, y_cm: f32) -> Placed {
    placed(
        id,
        PartSpec::Pin {
            x_cm,
            y_cm,
            angle_deg: -90.0,
        },
    )
}

pub fn funnel(
    id: &str,
    x_cm: f32,
    y_cm: f32,
    mouth_cm: f32,
    throat_cm: f32,
    height_cm: f32,
) -> Placed {
    placed(
        id,
        PartSpec::Funnel {
            x_cm,
            y_cm,
            mouth_cm,
            throat_cm,
            height_cm,
        },
    )
}

pub fn bucket(id: &str, x_cm: f32, y_cm: f32, width_cm: f32, height_cm: f32) -> Placed {
    placed(
        id,
        PartSpec::Bucket {
            x_cm,
            y_cm,
            width_cm,
            height_cm,
        },
    )
}

pub fn sim(level: &Level, scene: &Scene) -> Simulator {
    let valid =
        validate(level, scene, &Rules::V1).unwrap_or_else(|problems| panic!("{problems:#?}"));
    Simulator::new(level, &valid, &Rules::V1)
}

pub fn part(sim: &Simulator, name: &str) -> PartId {
    sim.part_id(name)
        .unwrap_or_else(|| panic!("no part named {name}"))
}

pub fn step_for(sim: &mut Simulator, seconds: f32) {
    let steps = (seconds / Rules::V1.dt_s).round() as u32;
    for _ in 0..steps {
        sim.step();
    }
}

pub fn close(actual: f32, expected: f32, tolerance: f32) -> bool {
    (actual - expected).abs() <= tolerance * expected.abs()
}

/// Centre of a ball resting on a plank whose centre line passes through (`x_cm`, `y_cm`) at `angle_deg`.
pub fn on_plank(x_cm: f32, y_cm: f32, angle_deg: f32, along_cm: f32, radius_m: f32) -> [f32; 2] {
    let a = angle_deg * PI / 180.0;
    let (cos, sin) = (libm::cosf(a), libm::sinf(a));
    let lift = (radius_m + Rules::V1.plank_thickness_m / 2.0) * 100.0 + 0.2;
    [
        x_cm + along_cm * cos - lift * sin,
        y_cm + along_cm * sin + lift * cos,
    ]
}
