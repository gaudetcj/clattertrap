//! Serde types for a level: fixed objects, build zones, inventory, goal, time limit.
//!
//! A level's reference solution lives in a separate file and never in this type.

use crate::scene::Placed;
use serde::{Deserialize, Serialize};

/// One level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    /// Stable id, also the directory name.
    pub id: String,
    /// Short title shown to viewers.
    pub title: String,
    /// Difficulty from 1 (one part) to 5 (several mechanisms in a chain).
    pub tier: u8,
    /// The layout in prose for contestants: where things are and what they do at the start.
    pub description: String,
    /// Rules version the level was authored for.
    pub rules_version: u32,
    /// Size of the world rectangle; the floor is its bottom edge.
    pub world: WorldSize,
    /// Run length cap.
    pub time_limit_s: f32,
    /// Objects placed by the level, including goal objects.
    pub fixed: Vec<Placed>,
    /// Rectangles where a contestant may place part centres.
    pub zones: Vec<Zone>,
    /// How many of each part kind a contestant may use.
    pub inventory: Inventory,
    /// What counts as success.
    pub goal: Goal,
    /// Whether chain length is scored too.
    #[serde(default)]
    pub showcase: bool,
}

/// World rectangle in centimetres, origin bottom-left.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSize {
    /// Width.
    pub width_cm: f32,
    /// Height.
    pub height_cm: f32,
}

/// A build zone rectangle in centimetres, by its bottom-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    /// Left edge.
    pub x_cm: f32,
    /// Bottom edge.
    pub y_cm: f32,
    /// Width.
    pub width_cm: f32,
    /// Height.
    pub height_cm: f32,
}

impl Zone {
    /// Whether a centimetre point lies inside or on the edge of the zone.
    #[must_use]
    pub fn contains_cm(&self, [x, y]: [f32; 2]) -> bool {
        x >= self.x_cm
            && x <= self.x_cm + self.width_cm
            && y >= self.y_cm
            && y <= self.y_cm + self.height_cm
    }
}

/// Parts a contestant may place, by kind. Missing kinds are zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Inventory {
    /// Beams.
    pub beam: u8,
    /// Funnels.
    pub funnel: u8,
    /// Balls of any material.
    pub ball: u8,
    /// Dominoes.
    pub domino: u8,
    /// Crates.
    #[serde(rename = "crate")]
    pub crates: u8,
    /// Seesaws.
    pub seesaw: u8,
    /// Pendulums.
    pub pendulum: u8,
    /// Spring pads.
    pub spring_pad: u8,
    /// Buckets.
    pub bucket: u8,
    /// Balloons.
    pub balloon: u8,
    /// Pins.
    pub pin: u8,
    /// Ropes.
    pub rope: u8,
}

impl Inventory {
    /// How many parts of a JSON `kind` the contestant may place; goal objects are never allowed.
    #[must_use]
    pub const fn allowed(&self, kind: &str) -> u8 {
        match kind.as_bytes() {
            b"beam" => self.beam,
            b"funnel" => self.funnel,
            b"ball" => self.ball,
            b"domino" => self.domino,
            b"crate" => self.crates,
            b"seesaw" => self.seesaw,
            b"pendulum" => self.pendulum,
            b"spring_pad" => self.spring_pad,
            b"bucket" => self.bucket,
            b"balloon" => self.balloon,
            b"pin" => self.pin,
            b"rope" => self.rope,
            _ => 0,
        }
    }
}

/// The level's goal. `target` names a fixed object; `via` lists parts that must be touched first, in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Goal {
    /// Ring the bell named by `target`.
    Ring {
        /// Bell id.
        target: String,
        /// Waypoint part ids.
        #[serde(default)]
        via: Vec<String>,
    },
    /// Press the button named by `target`.
    Press {
        /// Button id.
        target: String,
        /// Waypoint part ids.
        #[serde(default)]
        via: Vec<String>,
    },
    /// Rest `count` objects inside the basket named by `target`.
    Fill {
        /// Basket id.
        target: String,
        /// Objects required at rest inside.
        count: u32,
        /// Waypoint part ids.
        #[serde(default)]
        via: Vec<String>,
    },
    /// Pop the balloon named by `target`.
    Pop {
        /// Balloon id.
        target: String,
        /// Waypoint part ids.
        #[serde(default)]
        via: Vec<String>,
    },
}

impl Goal {
    /// Id of the goal object.
    #[must_use]
    pub fn target(&self) -> &str {
        match self {
            Self::Ring { target, .. }
            | Self::Press { target, .. }
            | Self::Fill { target, .. }
            | Self::Pop { target, .. } => target,
        }
    }

    /// Waypoint part ids that must be contacted in order before the goal counts.
    #[must_use]
    pub fn via(&self) -> &[String] {
        match self {
            Self::Ring { via, .. }
            | Self::Press { via, .. }
            | Self::Fill { via, .. }
            | Self::Pop { via, .. } => via,
        }
    }
}
