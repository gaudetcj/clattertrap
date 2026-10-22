//! Serde types for a contestant's placement. Field names carry their units (`x_cm`, `angle_deg`).
//!
//! The same `PartSpec` describes a level's fixed objects; the goal objects `bell`, `button`, and
//! `basket` are only accepted there, which `validate` enforces.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A contestant's placement for one level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// Id of the level this scene is for.
    pub level: String,
    /// Placed parts, in the order the contestant listed them.
    pub parts: Vec<Placed>,
}

/// One part with its id. The `kind` tag and the part's fields sit beside `id` in JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Placed {
    /// Unique id within the level and scene.
    pub id: String,
    /// What the part is and where it goes.
    #[serde(flatten)]
    pub spec: PartSpec,
}

/// Ball materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    /// Heavy, barely bounces.
    Steel,
    /// Medium weight and bounce.
    Wood,
    /// Light, bounces high.
    Rubber,
}

const fn pin_down() -> f32 {
    -90.0
}

/// Every part kind of rules v1 with its parameters. Positions are the part's centre in centimetres,
/// y up; angles are degrees, counterclockwise positive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PartSpec {
    /// Fixed wooden plank.
    Beam {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
        /// Length along the plank.
        length_cm: f32,
        /// Tilt; negative slopes down to the right.
        angle_deg: f32,
    },
    /// Fixed pair of inward-sloping walls.
    Funnel {
        /// Centre x of the throat.
        x_cm: f32,
        /// y of the throat opening.
        y_cm: f32,
        /// Width of the top opening.
        mouth_cm: f32,
        /// Width of the bottom opening.
        throat_cm: f32,
        /// Height from throat to mouth.
        height_cm: f32,
    },
    /// Rolling ball of a material.
    Ball {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
        /// Steel, wood, or rubber.
        material: Material,
    },
    /// Standing tile that tips when pushed.
    Domino {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
        /// Tilt from upright.
        #[serde(default)]
        angle_deg: f32,
    },
    /// Wooden box.
    Crate {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
        /// Side length.
        side_cm: f32,
    },
    /// Plank pivoting freely about its fixed centre.
    Seesaw {
        /// Pivot x.
        x_cm: f32,
        /// Pivot y.
        y_cm: f32,
        /// Plank length.
        length_cm: f32,
        /// Starting tilt.
        #[serde(default)]
        angle_deg: f32,
    },
    /// Steel weight hanging from a fixed pin on a rigid rope.
    Pendulum {
        /// Pin x.
        x_cm: f32,
        /// Pin y.
        y_cm: f32,
        /// Rope length.
        length_cm: f32,
        /// Starting angle from straight down, counterclockwise positive.
        #[serde(default)]
        angle_deg: f32,
    },
    /// Fixed pad that launches whatever lands on it.
    SpringPad {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
        /// Tilt of the pad; the launch is along its up normal.
        #[serde(default)]
        angle_deg: f32,
    },
    /// Open-top wooden container that moves.
    Bucket {
        /// Centre x of the floor.
        x_cm: f32,
        /// y of the floor's underside.
        y_cm: f32,
        /// Inner width.
        width_cm: f32,
        /// Inner height.
        height_cm: f32,
    },
    /// Rises until it meets a pin.
    Balloon {
        /// Centre x.
        x_cm: f32,
        /// Centre y.
        y_cm: f32,
    },
    /// Fixed needle that pops balloons.
    Pin {
        /// Base x.
        x_cm: f32,
        /// Base y.
        y_cm: f32,
        /// Direction the point faces; the default points down.
        #[serde(default = "pin_down")]
        angle_deg: f32,
    },
    /// Links two parts so they stay within a maximum distance.
    Rope {
        /// Id of the first linked part.
        a: String,
        /// Id of the second linked part.
        b: String,
        /// Longest the rope can stretch.
        max_length_cm: f32,
    },
    /// A pair of portals a ball passes between, placed only by a level.
    Portal {
        /// Orange end centre x.
        x_cm: f32,
        /// Orange end centre y.
        y_cm: f32,
        /// Direction the orange end faces, out into the room.
        angle_deg: f32,
        /// Blue end centre x.
        blue_x_cm: f32,
        /// Blue end centre y.
        blue_y_cm: f32,
        /// Direction the blue end faces, out into the room.
        blue_angle_deg: f32,
    },
    /// Goal object: brass bell hanging from a bracket.
    Bell {
        /// Bell centre x.
        x_cm: f32,
        /// Bell centre y.
        y_cm: f32,
        /// Rope length up to the bracket.
        rope_cm: f32,
    },
    /// Goal object: a cap that must be pushed down.
    Button {
        /// Centre x.
        x_cm: f32,
        /// y of the cap's resting underside.
        y_cm: f32,
        /// Travel needed to press.
        travel_cm: f32,
    },
    /// Goal object: fixed open-top container to fill.
    Basket {
        /// Centre x of the floor.
        x_cm: f32,
        /// y of the floor's underside.
        y_cm: f32,
        /// Inner width.
        inner_width_cm: f32,
        /// Inner height.
        inner_height_cm: f32,
    },
}

impl PartSpec {
    /// The JSON `kind` tag of this part.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Beam { .. } => "beam",
            Self::Funnel { .. } => "funnel",
            Self::Ball { .. } => "ball",
            Self::Domino { .. } => "domino",
            Self::Crate { .. } => "crate",
            Self::Seesaw { .. } => "seesaw",
            Self::Pendulum { .. } => "pendulum",
            Self::SpringPad { .. } => "spring_pad",
            Self::Bucket { .. } => "bucket",
            Self::Balloon { .. } => "balloon",
            Self::Pin { .. } => "pin",
            Self::Rope { .. } => "rope",
            Self::Portal { .. } => "portal",
            Self::Bell { .. } => "bell",
            Self::Button { .. } => "button",
            Self::Basket { .. } => "basket",
        }
    }

    /// Whether this is a goal object: a bell, button, or basket.
    #[must_use]
    pub const fn is_goal_object(&self) -> bool {
        matches!(
            self,
            Self::Bell { .. } | Self::Button { .. } | Self::Basket { .. }
        )
    }

    /// Whether only a level may place this kind: the goal objects and portals.
    #[must_use]
    pub const fn is_level_only(&self) -> bool {
        self.is_goal_object() || matches!(self, Self::Portal { .. })
    }

    /// Whether the part has a body that moves: a ball, domino, crate, seesaw, pendulum, or balloon.
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        matches!(
            self,
            Self::Ball { .. }
                | Self::Domino { .. }
                | Self::Crate { .. }
                | Self::Seesaw { .. }
                | Self::Pendulum { .. }
                | Self::Balloon { .. }
        )
    }

    /// The part's centre in centimetres, or `None` for a rope.
    #[must_use]
    pub const fn centre_cm(&self) -> Option<[f32; 2]> {
        match self {
            Self::Beam { x_cm, y_cm, .. }
            | Self::Funnel { x_cm, y_cm, .. }
            | Self::Ball { x_cm, y_cm, .. }
            | Self::Domino { x_cm, y_cm, .. }
            | Self::Crate { x_cm, y_cm, .. }
            | Self::Seesaw { x_cm, y_cm, .. }
            | Self::Pendulum { x_cm, y_cm, .. }
            | Self::SpringPad { x_cm, y_cm, .. }
            | Self::Bucket { x_cm, y_cm, .. }
            | Self::Balloon { x_cm, y_cm }
            | Self::Pin { x_cm, y_cm, .. }
            | Self::Portal { x_cm, y_cm, .. }
            | Self::Bell { x_cm, y_cm, .. }
            | Self::Button { x_cm, y_cm, .. }
            | Self::Basket { x_cm, y_cm, .. } => Some([*x_cm, *y_cm]),
            Self::Rope { .. } => None,
        }
    }
}

/// The JSON Schema for a scene, generated from these types.
#[must_use]
pub fn schema() -> serde_json::Value {
    schemars::schema_for!(Scene).to_value()
}
