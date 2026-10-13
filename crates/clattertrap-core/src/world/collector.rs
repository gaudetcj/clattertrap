//! Collects rapier's collision and contact force events into part-id pairs during a step.

use crate::events::PartId;
use rapier2d::prelude::{
    ColliderHandle, ColliderSet, CollisionEvent, ContactPair, EventHandler, RigidBodySet,
    SoftBodySet, SoftBodyTearEvent,
};
use std::sync::{Mutex, PoisonError};

/// Raw events of one step, in part ids. Sorted later by `StepEvents`.
#[derive(Debug, Default)]
pub(crate) struct RawEvents {
    /// Pairs that started touching.
    pub contacts: Vec<(PartId, PartId)>,
    /// Pairs whose summed contact impulse crossed a collider's threshold, with that impulse.
    pub impulses: Vec<(PartId, PartId, f32)>,
}

/// `EventHandler` backed by mutexes, because rapier requires `Sync`; locked once per event, drained once per step.
#[derive(Debug, Default)]
pub(crate) struct Collector {
    raw: Mutex<RawEvents>,
}

#[allow(clippy::cast_possible_truncation)]
fn part_of(colliders: &ColliderSet, handle: ColliderHandle) -> Option<PartId> {
    colliders.get(handle).map(|c| PartId(c.user_data as u16))
}

impl Collector {
    /// Moves this step's events into `out`, leaving the collector empty.
    pub(crate) fn drain_into(&self, out: &mut RawEvents) {
        let mut raw = self.raw.lock().unwrap_or_else(PoisonError::into_inner);
        out.contacts.clear();
        out.impulses.clear();
        out.contacts.append(&mut raw.contacts);
        out.impulses.append(&mut raw.impulses);
    }
}

impl EventHandler for Collector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        colliders: &ColliderSet,
        event: CollisionEvent,
        _pair: Option<&ContactPair>,
    ) {
        if let CollisionEvent::Started(h1, h2, _) = event
            && let (Some(a), Some(b)) = (part_of(colliders, h1), part_of(colliders, h2))
        {
            self.raw
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .contacts
                .push((a, b));
        }
    }

    fn handle_contact_force_event(
        &self,
        dt: f32,
        _bodies: &RigidBodySet,
        colliders: &ColliderSet,
        pair: &ContactPair,
        total_force_magnitude: f32,
    ) {
        if let (Some(a), Some(b)) = (
            part_of(colliders, pair.collider1),
            part_of(colliders, pair.collider2),
        ) {
            self.raw
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .impulses
                .push((a, b, total_force_magnitude * dt));
        }
    }

    fn handle_soft_body_tear_event(&self, _soft_bodies: &SoftBodySet, _event: &SoftBodyTearEvent) {}
}
