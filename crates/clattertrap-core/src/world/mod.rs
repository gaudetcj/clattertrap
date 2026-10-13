//! Owns the rapier pipeline. Nothing outside this module touches rapier types.

mod builder;
mod collector;

pub(crate) use builder::{Builder, Surface};
pub(crate) use collector::RawEvents;

use crate::events::PartId;
use crate::hash::Fnv64;
use crate::level::Level;
use crate::parts::{self, Look, Visual};
use crate::rules::Rules;
use crate::scene::PartSpec;
use crate::validate::ValidScene;
use builder::BodyMeta;
use collector::Collector;
use rapier2d::prelude::{PhysicsWorld, RigidBody, Vector};

/// Where one body is after a step, in metres and radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyPose {
    /// Part the body belongs to.
    pub part: PartId,
    /// Centre x.
    pub x_m: f32,
    /// Centre y.
    pub y_m: f32,
    /// Rotation, counterclockwise positive.
    pub angle_rad: f32,
    /// Whether the body is asleep.
    pub asleep: bool,
}

/// Penetration deeper than this at the start is an overlap; shallower is a resting contact.
const OVERLAP_TOLERANCE_M: f32 = 0.002;

/// Two parts that penetrate at the start, by part id with `a < b`, and how deep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Overlap {
    pub a: PartId,
    pub b: PartId,
    pub depth_m: f32,
}

/// The physics world for one run.
pub(crate) struct World {
    pw: PhysicsWorld,
    metas: Vec<BodyMeta>,
    collector: Collector,
    raw: RawEvents,
}

impl World {
    /// Builds the floor, the level's fixed objects, and the scene's parts, in that order.
    pub(crate) fn build(level: &Level, valid: &ValidScene, rules: &Rules) -> Self {
        let mut pw = PhysicsWorld::new();
        pw.gravity = Vector::new(0.0, -rules.gravity_m_s2);
        pw.integration_parameters.dt = rules.dt_s;
        pw.integration_parameters.max_ccd_substeps = rules.max_ccd_substeps;
        let mut metas = Vec::with_capacity(valid.placed().len() + 1);
        let mut b = Builder::new(&mut pw, &mut metas, rules, valid.names());
        let width_m = parts::m(level.world.width_cm);
        let floor = Surface {
            friction: rules.wood.friction,
            restitution: 0.0,
        };
        b.fixed_box(
            valid.floor_id(),
            [width_m / 2.0, -0.5],
            [width_m / 2.0 + rules.leave_margin_m, 0.5],
            0.0,
            floor,
            Look::Wood,
        );
        // Ropes link bodies, so every other part is built first.
        for pass in [false, true] {
            for (i, placed) in valid.placed().iter().enumerate() {
                if matches!(placed.spec, PartSpec::Rope { .. }) != pass {
                    continue;
                }
                // validate caps the part count at u16::MAX.
                #[allow(clippy::cast_possible_truncation)]
                parts::build(&placed.spec, PartId(i as u16), &mut b);
            }
        }
        Self {
            pw,
            metas,
            collector: Collector::default(),
            raw: RawEvents::default(),
        }
    }

    /// Runs collision detection without stepping and reports every pair that already penetrates.
    pub(crate) fn initial_overlaps(&mut self) -> Vec<Overlap> {
        self.pw.detect_collisions(&(), &());
        let part_of = |h| {
            self.pw
                .colliders
                .get(h)
                .and_then(|c| u16::try_from(c.user_data).ok())
                .map(PartId)
        };
        let mut found = Vec::new();
        for pair in self.pw.contact_pairs() {
            let Some((_, contact)) = pair.find_deepest_contact() else {
                continue;
            };
            if contact.dist >= -OVERLAP_TOLERANCE_M {
                continue;
            }
            if let (Some(a), Some(b)) = (part_of(pair.collider1), part_of(pair.collider2)) {
                found.push(Overlap {
                    a,
                    b,
                    depth_m: -contact.dist,
                });
            }
        }
        found.sort_by_key(|o| (o.a, o.b));
        found
    }

    /// Advances one fixed step and returns the raw events it produced.
    pub(crate) fn step(&mut self) -> &RawEvents {
        self.pw.step_with_events(&(), &self.collector);
        self.collector.drain_into(&mut self.raw);
        &self.raw
    }

    fn body(&self, part: PartId) -> Option<&RigidBody> {
        self.metas
            .iter()
            .find(|m| m.part == part)
            .and_then(|m| self.pw.bodies.get(m.handle))
    }

    fn pose_of(part: PartId, body: &RigidBody) -> BodyPose {
        let t = body.translation();
        BodyPose {
            part,
            x_m: t.x,
            y_m: t.y,
            angle_rad: body.rotation().angle(),
            asleep: body.is_sleeping(),
        }
    }

    /// Every body's pose, in insertion order.
    pub(crate) fn poses(&self) -> impl Iterator<Item = BodyPose> + '_ {
        self.metas.iter().filter_map(|m| {
            self.pw
                .bodies
                .get(m.handle)
                .map(|b| Self::pose_of(m.part, b))
        })
    }

    /// Every body's pose with its drawing description, in insertion order.
    pub(crate) fn bodies(&self) -> impl Iterator<Item = (BodyPose, Visual)> + '_ {
        self.metas.iter().filter_map(|m| {
            self.pw
                .bodies
                .get(m.handle)
                .map(|b| (Self::pose_of(m.part, b), m.visual))
        })
    }

    /// The pose of a part's first body.
    pub(crate) fn pose(&self, part: PartId) -> Option<BodyPose> {
        self.body(part).map(|b| Self::pose_of(part, b))
    }

    /// Linear velocity of a part's first body.
    pub(crate) fn velocity(&self, part: PartId) -> Option<[f32; 2]> {
        self.body(part).map(|b| {
            let v = b.linvel();
            [v.x, v.y]
        })
    }

    /// Angular velocity of a part's first body.
    pub(crate) fn angular_velocity(&self, part: PartId) -> Option<f32> {
        self.body(part).map(RigidBody::angvel)
    }

    /// Whether no dynamic body is awake.
    pub(crate) fn all_asleep(&self) -> bool {
        self.metas.iter().filter(|m| m.dynamic).all(|m| {
            self.pw
                .bodies
                .get(m.handle)
                .is_none_or(RigidBody::is_sleeping)
        })
    }

    /// FNV-1a 64 over every body's pose, velocities, and sleep flag, in insertion order.
    pub(crate) fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        for m in &self.metas {
            let Some(b) = self.pw.bodies.get(m.handle) else {
                h.write_u8(0xff);
                continue;
            };
            h.write_u16(m.part.0);
            let t = b.translation();
            h.write_f32(t.x);
            h.write_f32(t.y);
            h.write_f32(b.rotation().angle());
            let v = b.linvel();
            h.write_f32(v.x);
            h.write_f32(v.y);
            h.write_f32(b.angvel());
            h.write_u8(u8::from(b.is_sleeping()));
        }
        h.finish()
    }
}
