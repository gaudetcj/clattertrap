//! Events the simulator emits, and the per-step buffer that sorts them by a stable key.

use crate::hash::Fnv64;

/// Index of a part in the validated scene's name table. Fixed objects come first, then the
/// contestant's parts, then the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartId(pub u16);

/// What happened. The derived ordering is the stable sort key: kind first, then part ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKind {
    /// Two parts started touching; `a` is the smaller id.
    Contact {
        /// Smaller part id.
        a: PartId,
        /// Larger part id.
        b: PartId,
    },
    /// A goal waypoint was touched in order.
    Via(PartId),
    /// The goal was met.
    GoalMet,
    /// A part fell asleep.
    AtRest(PartId),
    /// A part left the world rectangle and was removed.
    LeftWorld(PartId),
}

impl EventKind {
    /// A contact with its ids normalised so the smaller comes first.
    #[must_use]
    pub fn contact(a: PartId, b: PartId) -> Self {
        if a <= b {
            Self::Contact { a, b }
        } else {
            Self::Contact { a: b, b: a }
        }
    }

    fn write_to(self, h: &mut Fnv64) {
        match self {
            Self::Contact { a, b } => {
                h.write_u8(0);
                h.write_u16(a.0);
                h.write_u16(b.0);
            }
            Self::Via(p) => {
                h.write_u8(1);
                h.write_u16(p.0);
            }
            Self::GoalMet => h.write_u8(2),
            Self::AtRest(p) => {
                h.write_u8(3);
                h.write_u16(p.0);
            }
            Self::LeftWorld(p) => {
                h.write_u8(4);
                h.write_u16(p.0);
            }
        }
    }
}

/// One event at a step. Simulated time is `step * dt_s`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    /// Step index after which the event was observed, counting from 1.
    pub step: u32,
    /// What happened.
    pub kind: EventKind,
}

impl Event {
    /// Feeds the event into a hash.
    pub fn write_to(self, h: &mut Fnv64) {
        h.write_u32(self.step);
        self.kind.write_to(h);
    }
}

/// Reused buffer for one step's events. `finish_step` sorts and deduplicates.
#[derive(Debug, Default)]
pub struct StepEvents {
    buf: Vec<Event>,
}

impl StepEvents {
    /// Empty buffer with room for a busy step.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(64),
        }
    }

    /// Adds an event observed during the current step.
    pub fn push(&mut self, kind: EventKind) {
        self.buf.push(Event { step: 0, kind });
    }

    /// Stamps every buffered event with `step`, sorts by the stable key, and drops duplicates.
    pub fn finish_step(&mut self, step: u32) {
        for e in &mut self.buf {
            e.step = step;
        }
        self.buf.sort_unstable();
        self.buf.dedup();
    }

    /// The finished step's events.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.buf
    }

    /// Forgets the buffered events, keeping the allocation.
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, EventKind, PartId, StepEvents};

    #[test]
    fn contacts_are_normalised_so_a_is_the_smaller_id() {
        assert_eq!(
            EventKind::contact(PartId(3), PartId(1)),
            EventKind::Contact {
                a: PartId(1),
                b: PartId(3)
            }
        );
    }

    #[test]
    fn a_step_sorts_by_kind_then_ids_and_drops_duplicates() {
        let mut buf = StepEvents::new();
        buf.push(EventKind::AtRest(PartId(0)));
        buf.push(EventKind::contact(PartId(2), PartId(1)));
        buf.push(EventKind::GoalMet);
        buf.push(EventKind::contact(PartId(1), PartId(2)));
        buf.push(EventKind::contact(PartId(0), PartId(5)));
        buf.finish_step(7);
        let kinds: Vec<EventKind> = buf.events().iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            [
                EventKind::Contact {
                    a: PartId(0),
                    b: PartId(5)
                },
                EventKind::Contact {
                    a: PartId(1),
                    b: PartId(2)
                },
                EventKind::GoalMet,
                EventKind::AtRest(PartId(0)),
            ]
        );
        assert!(buf.events().iter().all(|e| e.step == 7));
        buf.clear();
        assert!(buf.events().is_empty());
    }

    #[test]
    fn events_compare_by_step_first() {
        let early = Event {
            step: 1,
            kind: EventKind::GoalMet,
        };
        let late = Event {
            step: 2,
            kind: EventKind::contact(PartId(0), PartId(1)),
        };
        assert!(early < late);
    }
}
