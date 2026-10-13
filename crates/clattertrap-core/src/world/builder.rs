//! Adds bodies, colliders, and joints to the world under construction, tagging each body with its part id.

use crate::events::PartId;
use crate::parts::{Look, Visual};
use crate::rules::{MaterialProps, Rules};
use rapier2d::prelude::{
    ActiveEvents, CoefficientCombineRule, ColliderBuilder, MotorModel, PhysicsWorld,
    PrismaticJointBuilder, RevoluteJointBuilder, RigidBodyBuilder, RigidBodyHandle,
    RopeJointBuilder, Vector,
};

/// Friction and restitution of a fixed surface.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Surface {
    /// Coulomb friction.
    pub friction: f32,
    /// Restitution.
    pub restitution: f32,
}

/// How a balloon floats: a negative gravity scale and the damping that caps its rise.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Lift {
    pub density: f32,
    pub gravity_scale: f32,
    pub damping: f32,
}

/// Inner size and wall thickness of an open-top U.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Trough {
    pub inner_m: [f32; 2],
    pub wall_m: f32,
}

/// A sprung slider: its axis, travel limits, and the spring that returns it to zero.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Slide {
    pub axis: [f32; 2],
    pub limits_m: [f32; 2],
    pub stiffness_n_m: f32,
    pub damping: f32,
}

/// One rigid body, the part it belongs to, and how to draw it, in insertion order.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BodyMeta {
    pub handle: RigidBodyHandle,
    pub part: PartId,
    pub dynamic: bool,
    pub visual: Visual,
}

/// Builder handed to each part's `build`.
pub(crate) struct Builder<'a> {
    pw: &'a mut PhysicsWorld,
    metas: &'a mut Vec<BodyMeta>,
    rules: &'a Rules,
    names: &'a [String],
}

impl<'a> Builder<'a> {
    pub(crate) fn new(
        pw: &'a mut PhysicsWorld,
        metas: &'a mut Vec<BodyMeta>,
        rules: &'a Rules,
        names: &'a [String],
    ) -> Self {
        Self {
            pw,
            metas,
            rules,
            names,
        }
    }

    /// The rules in force.
    pub(crate) fn rules(&self) -> &Rules {
        self.rules
    }

    /// The id of a named part.
    pub(crate) fn part_id(&self, name: &str) -> Option<PartId> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(PartId)
    }

    /// The first body built for a part.
    pub(crate) fn first_body(&self, part: PartId) -> Option<RigidBodyHandle> {
        self.metas.iter().find(|m| m.part == part).map(|m| m.handle)
    }

    /// A fixed rectangle centred at `centre_m` with the given half extents and rotation.
    pub(crate) fn fixed_box(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        half_m: [f32; 2],
        angle_rad: f32,
        surface: Surface,
        look: Look,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::fixed()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .rotation(angle_rad);
        let collider = ColliderBuilder::cuboid(half_m[0], half_m[1])
            .friction(surface.friction)
            .restitution(surface.restitution);
        let visual = Visual::Box {
            w_m: 2.0 * half_m[0],
            h_m: 2.0 * half_m[1],
            look,
        };
        self.insert(part, false, body, collider, visual)
    }

    /// A fixed body with no collider, for pivots and pins.
    pub(crate) fn fixed_anchor(&mut self, part: PartId, point_m: [f32; 2]) -> RigidBodyHandle {
        let body = RigidBodyBuilder::fixed().translation(Vector::new(point_m[0], point_m[1]));
        let handle = self.pw.insert_body(body);
        self.metas.push(BodyMeta {
            handle,
            part,
            dynamic: false,
            visual: Visual::Pivot,
        });
        handle
    }

    /// A dynamic disc with continuous collision detection.
    pub(crate) fn dynamic_ball(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        radius_m: f32,
        props: MaterialProps,
        look: Look,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .ccd_enabled(true);
        let collider = ColliderBuilder::ball(radius_m)
            .density(props.density)
            .friction(props.friction)
            .restitution(props.restitution);
        self.insert(
            part,
            true,
            body,
            collider,
            Visual::Circle {
                r_m: radius_m,
                look,
            },
        )
    }

    /// A dynamic rectangle with continuous collision detection.
    pub(crate) fn dynamic_box(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        half_m: [f32; 2],
        angle_rad: f32,
        props: MaterialProps,
        look: Look,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .rotation(angle_rad)
            .ccd_enabled(true);
        let collider = ColliderBuilder::cuboid(half_m[0], half_m[1])
            .density(props.density)
            .friction(props.friction)
            .restitution(props.restitution);
        let visual = Visual::Box {
            w_m: 2.0 * half_m[0],
            h_m: 2.0 * half_m[1],
            look,
        };
        self.insert(part, true, body, collider, visual)
    }

    /// A rising disc: light, pushed up by a negative gravity scale, slowed by damping.
    pub(crate) fn balloon(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        radius_m: f32,
        lift: Lift,
        look: Look,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .gravity_scale(lift.gravity_scale)
            .linear_damping(lift.damping)
            .ccd_enabled(true);
        let collider = ColliderBuilder::ball(radius_m)
            .density(lift.density)
            .friction(0.1)
            .restitution(0.1);
        self.insert(
            part,
            true,
            body,
            collider,
            Visual::Circle {
                r_m: radius_m,
                look,
            },
        )
    }

    /// An open-top U: a floor and two walls on one body whose origin is the floor's underside centre.
    pub(crate) fn trough(
        &mut self,
        part: PartId,
        dynamic: bool,
        origin_m: [f32; 2],
        shape: Trough,
        props: MaterialProps,
        look: Look,
    ) -> RigidBodyHandle {
        let builder = if dynamic {
            RigidBodyBuilder::dynamic().ccd_enabled(true)
        } else {
            RigidBodyBuilder::fixed()
        };
        let handle = self
            .pw
            .insert_body(builder.translation(Vector::new(origin_m[0], origin_m[1])));
        let Trough {
            inner_m: [w, h],
            wall_m,
        } = shape;
        let wall_x = f32::midpoint(w, wall_m);
        let pieces = [
            ([0.0, wall_m / 2.0], [w / 2.0 + wall_m, wall_m / 2.0]),
            ([-wall_x, wall_m + h / 2.0], [wall_m / 2.0, h / 2.0]),
            ([wall_x, wall_m + h / 2.0], [wall_m / 2.0, h / 2.0]),
        ];
        for (at, half) in pieces {
            let collider = ColliderBuilder::cuboid(half[0], half[1])
                .translation(Vector::new(at[0], at[1]))
                .density(props.density)
                .friction(props.friction)
                .restitution(props.restitution)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .active_events(ActiveEvents::COLLISION_EVENTS)
                .user_data(u128::from(part.0));
            self.pw.insert_collider(collider, Some(handle));
        }
        let visual = Visual::Trough {
            inner_w_m: w,
            inner_h_m: h,
            wall_m,
            look,
        };
        self.metas.push(BodyMeta {
            handle,
            part,
            dynamic,
            visual,
        });
        handle
    }

    /// A dynamic disc that also reports contact forces above `force_threshold_n`, for the bell.
    pub(crate) fn bell(
        &mut self,
        part: PartId,
        centre_m: [f32; 2],
        radius_m: f32,
        props: MaterialProps,
        force_threshold_n: f32,
        look: Look,
    ) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(centre_m[0], centre_m[1]))
            .ccd_enabled(true);
        let collider = ColliderBuilder::ball(radius_m)
            .density(props.density)
            .friction(props.friction)
            .restitution(props.restitution)
            .active_events(ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS)
            .contact_force_event_threshold(force_threshold_n);
        self.insert(
            part,
            true,
            body,
            collider,
            Visual::Circle {
                r_m: radius_m,
                look,
            },
        )
    }

    /// A slider along `axis` between two bodies, held at zero by a spring and stopped at `limits`.
    pub(crate) fn prismatic(
        &mut self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        anchors_m: [[f32; 2]; 2],
        slide: Slide,
    ) {
        let joint = PrismaticJointBuilder::new(Vector::new(slide.axis[0], slide.axis[1]))
            .local_anchor1(Vector::new(anchors_m[0][0], anchors_m[0][1]))
            .local_anchor2(Vector::new(anchors_m[1][0], anchors_m[1][1]))
            .limits(slide.limits_m)
            .motor_model(MotorModel::ForceBased)
            .motor_position(0.0, slide.stiffness_n_m, slide.damping);
        self.pw.insert_impulse_joint(a, b, joint);
    }

    /// A free hinge between two bodies at the given body-local anchors.
    pub(crate) fn revolute(
        &mut self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        first_anchor_m: [f32; 2],
        second_anchor_m: [f32; 2],
    ) {
        let joint = RevoluteJointBuilder::new()
            .local_anchor1(Vector::new(first_anchor_m[0], first_anchor_m[1]))
            .local_anchor2(Vector::new(second_anchor_m[0], second_anchor_m[1]));
        self.pw.insert_impulse_joint(a, b, joint);
    }

    /// A rope between two body centres that cannot stretch past `max_m`.
    pub(crate) fn rope(&mut self, a: RigidBodyHandle, b: RigidBodyHandle, max_m: f32) {
        self.pw
            .insert_impulse_joint(a, b, RopeJointBuilder::new(max_m));
    }

    fn insert(
        &mut self,
        part: PartId,
        dynamic: bool,
        body: RigidBodyBuilder,
        collider: ColliderBuilder,
        visual: Visual,
    ) -> RigidBodyHandle {
        // Restitution takes the bouncier of the pair so a rubber ball bounces on a dead floor.
        let events = collider.active_events | ActiveEvents::COLLISION_EVENTS;
        let collider = collider
            .restitution_combine_rule(CoefficientCombineRule::Max)
            .active_events(events)
            .user_data(u128::from(part.0));
        let (handle, _) = self.pw.insert(body, collider);
        self.metas.push(BodyMeta {
            handle,
            part,
            dynamic,
            visual,
        });
        handle
    }
}
