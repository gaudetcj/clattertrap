//! `RULES_VERSION` and every physical and scoring constant. Nothing else defines constants.
//!
//! Lengths are metres, masses kilograms, times seconds, impulses newton seconds. Two-dimensional
//! densities are kilograms per square metre. See docs/physics.md.

use crate::scene::Material;

/// Version of the physics and scoring rules. Stays 1 until the first gallery entry is committed.
pub const RULES_VERSION: u32 = 1;

/// Physical properties of one material.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialProps {
    /// Area density in kg/m².
    pub density: f32,
    /// Coefficient of restitution, 0 to 1.
    pub restitution: f32,
    /// Coulomb friction coefficient.
    pub friction: f32,
}

/// An inclusive range of lengths a part parameter may take, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    /// Smallest allowed value.
    pub min_m: f32,
    /// Largest allowed value.
    pub max_m: f32,
}

/// Every constant the simulator and scorer use. Core functions take a `&Rules`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rules {
    /// Rules version these constants belong to.
    pub version: u32,
    /// Fixed time step.
    pub dt_s: f32,
    /// Downward gravity.
    pub gravity_m_s2: f32,
    /// Continuous collision detection substeps per step.
    pub max_ccd_substeps: usize,
    /// Steel: heavy, barely bounces.
    pub steel: MaterialProps,
    /// Wood: medium weight and bounce.
    pub wood: MaterialProps,
    /// Rubber: light, bounces high.
    pub rubber: MaterialProps,
    /// Radius of every ball.
    pub ball_radius_m: f32,
    /// Thickness of beams and seesaw planks.
    pub plank_thickness_m: f32,
    /// Allowed beam lengths.
    pub beam_length: Range,
    /// Allowed seesaw plank lengths.
    pub seesaw_length: Range,
    /// Domino width and height.
    pub domino_size_m: [f32; 2],
    /// Allowed crate side lengths.
    pub crate_side: Range,
    /// Radius of a pendulum weight (steel).
    pub pendulum_radius_m: f32,
    /// Allowed pendulum rope lengths.
    pub pendulum_length: Range,
    /// Spring pad width and thickness.
    pub spring_pad_size_m: [f32; 2],
    /// Impulse a spring pad gives a body that touches its top face.
    pub spring_impulse_ns: f32,
    /// Bucket and funnel wall thickness.
    pub wall_thickness_m: f32,
    /// Allowed bucket inner widths and heights.
    pub bucket_size: Range,
    /// Allowed funnel mouth widths; the throat must be narrower.
    pub funnel_mouth: Range,
    /// Allowed funnel heights.
    pub funnel_height: Range,
    /// Balloon radius.
    pub balloon_radius_m: f32,
    /// Balloon area density.
    pub balloon_density: f32,
    /// Balloon gravity scale; negative so it rises.
    pub balloon_gravity_scale: f32,
    /// Balloon linear damping, which sets its terminal rise speed.
    pub balloon_damping: f32,
    /// Pin width and length.
    pub pin_size_m: [f32; 2],
    /// Allowed rope lengths.
    pub rope_length: Range,
    /// Bell radius.
    pub bell_radius_m: f32,
    /// Bell area density.
    pub bell_density: f32,
    /// Smallest contact impulse that rings the bell.
    pub bell_min_impulse_ns: f32,
    /// Allowed bell rope lengths.
    pub bell_rope: Range,
    /// Button cap width and height.
    pub button_cap_size_m: [f32; 2],
    /// Allowed button travel.
    pub button_travel: Range,
    /// Narrowest funnel throat; two balls must still pass.
    pub funnel_throat_min_m: f32,
    /// Basket wall thickness.
    pub basket_wall_m: f32,
    /// Speed below which a body counts as resting for goals.
    pub rest_speed_m_s: f32,
    /// Speed above which a part counts as set moving for chain length.
    pub chain_move_speed_m_s: f32,
    /// Distance past the level rectangle at which a body has left the world.
    pub leave_margin_m: f32,
}

impl Rules {
    /// Rules v1.
    pub const V1: Rules = Rules {
        version: 1,
        dt_s: 1.0 / 120.0,
        gravity_m_s2: 9.81,
        max_ccd_substeps: 4,
        steel: MaterialProps {
            density: 88.0,
            restitution: 0.2,
            friction: 0.2,
        },
        wood: MaterialProps {
            density: 13.0,
            restitution: 0.3,
            friction: 0.5,
        },
        rubber: MaterialProps {
            density: 18.0,
            restitution: 0.8,
            friction: 0.9,
        },
        ball_radius_m: 0.06,
        plank_thickness_m: 0.04,
        beam_length: Range {
            min_m: 0.20,
            max_m: 1.50,
        },
        seesaw_length: Range {
            min_m: 0.40,
            max_m: 1.50,
        },
        domino_size_m: [0.04, 0.20],
        crate_side: Range {
            min_m: 0.10,
            max_m: 0.40,
        },
        pendulum_radius_m: 0.05,
        pendulum_length: Range {
            min_m: 0.20,
            max_m: 1.20,
        },
        spring_pad_size_m: [0.30, 0.04],
        spring_impulse_ns: 1.0,
        wall_thickness_m: 0.02,
        bucket_size: Range {
            min_m: 0.15,
            max_m: 0.60,
        },
        funnel_mouth: Range {
            min_m: 0.30,
            max_m: 1.50,
        },
        funnel_height: Range {
            min_m: 0.20,
            max_m: 0.80,
        },
        balloon_radius_m: 0.10,
        balloon_density: 1.0,
        balloon_gravity_scale: -0.5,
        balloon_damping: 3.3,
        pin_size_m: [0.01, 0.16],
        rope_length: Range {
            min_m: 0.10,
            max_m: 2.00,
        },
        bell_radius_m: 0.10,
        bell_density: 40.0,
        bell_min_impulse_ns: 0.1,
        bell_rope: Range {
            min_m: 0.10,
            max_m: 1.00,
        },
        button_cap_size_m: [0.12, 0.03],
        button_travel: Range {
            min_m: 0.01,
            max_m: 0.10,
        },
        funnel_throat_min_m: 0.14,
        basket_wall_m: 0.02,
        rest_speed_m_s: 0.02,
        chain_move_speed_m_s: 0.05,
        leave_margin_m: 0.5,
    };

    /// Properties of a ball material.
    #[must_use]
    pub const fn material(&self, material: Material) -> MaterialProps {
        match material {
            Material::Steel => self.steel,
            Material::Wood => self.wood,
            Material::Rubber => self.rubber,
        }
    }
}
