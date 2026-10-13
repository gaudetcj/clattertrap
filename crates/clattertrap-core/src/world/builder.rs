//! Adds bodies and colliders to the world under construction, tagging each with its part id.

use crate::events::PartId;
use crate::rules::{MaterialProps, Rules};
use rapier2d::prelude::{
    ActiveEvents, CoefficientCombineRule, ColliderBuilder, PhysicsWorld, RigidBodyBuilder,
    RigidBodyHandle, Vector,
};

/// Friction and restitution of a fixed surface.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Surface {
    /// Coulomb friction.
    pub friction: f32,
    /// Restitution.
    pub restitution: f32,
}

/// One rigid body and the part it belongs to, in insertion order.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BodyMeta {
    pub handle: RigidBodyHandle,
    pub part: PartId,
    pub dynamic: bool,
}

/// Builder handed to each part's `build`.
pub(crate) struct Builder<'a> {
    pw: &'a mut PhysicsWorld,
    metas: &'a mut Vec<BodyMeta>,
    rules: &'a Rules,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(
        pw: &'a mut PhysicsWorld,
        metas: &'a mut Vec<BodyMeta>,
        rules: &'a Rules,
    ) -> Self {
        Self { pw, metas, rules }
    }

    /// The rules in force.
    pub(crate) fn rules(&self) -> &Rules {
        self.rules
    }

    /// A fixed rectangle centred at `centre_m` with the given half extents and rotation.
    pub(crate) fn fixed_box(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        half_m: [f32; 2],
        angle_rad: f32,
        surface: Surface,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::fixed()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .rotation(angle_rad);
        let collider = ColliderBuilder::cuboid(half_m[0], half_m[1])
            .friction(surface.friction)
            .restitution(surface.restitution);
        self.insert(part, false, body, collider)
    }

    /// A dynamic disc with continuous collision detection.
    pub(crate) fn dynamic_ball(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        radius_m: f32,
        props: MaterialProps,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .ccd_enabled(true);
        let collider = ColliderBuilder::ball(radius_m)
            .density(props.density)
            .friction(props.friction)
            .restitution(props.restitution);
        self.insert(part, true, body, collider)
    }

    fn insert(
        &mut self,
        part: PartId,
        dynamic: bool,
        body: RigidBodyBuilder,
        collider: ColliderBuilder,
    ) -> RigidBodyHandle {
        // Restitution takes the bouncier of the pair so a rubber ball bounces on a dead floor.
        let collider = collider
            .restitution_combine_rule(CoefficientCombineRule::Max)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .user_data(u128::from(part.0));
        let (handle, _) = self.pw.insert(body, collider);
        self.metas.push(BodyMeta {
            handle,
            part,
            dynamic,
        });
        handle
    }
}
