//! Tracks a level's goal and its `via` waypoints as the run plays out.

use crate::events::{EventKind, PartId, StepEvents};
use crate::level::{Goal, Level};
use crate::parts::m;
use crate::rules::Rules;
use crate::scene::PartSpec;
use crate::validate::ValidScene;
use crate::world::World;

/// How much short of the full travel still counts as pressed, since the joint limit is soft.
const PRESS_SLACK_M: f32 = 0.002;

enum Kind {
    Ring,
    Press {
        rest_y_m: f32,
        cap_half_h_m: f32,
        travel_m: f32,
    },
    Fill {
        count: usize,
        inner_m: [f32; 4],
    },
    Pop,
}

/// The goal, resolved to part ids, with the waypoints touched so far.
pub(crate) struct Tracker {
    kind: Kind,
    target: PartId,
    via: Vec<PartId>,
    via_done: usize,
}

impl Tracker {
    /// Resolves the level's goal against the validated scene. `validate` guarantees the names exist.
    pub(crate) fn new(level: &Level, scene: &ValidScene, rules: &Rules) -> Self {
        let target = scene
            .part_id(level.goal.target())
            .unwrap_or(PartId(u16::MAX));
        let via = level
            .goal
            .via()
            .iter()
            .filter_map(|n| scene.part_id(n))
            .collect();
        let spec = scene.placed().get(usize::from(target.0)).map(|p| &p.spec);
        let kind = match (&level.goal, spec) {
            (
                Goal::Press { .. },
                Some(PartSpec::Button {
                    y_cm, travel_cm, ..
                }),
            ) => Kind::Press {
                rest_y_m: m(*y_cm),
                cap_half_h_m: rules.button_cap_size_m[1] / 2.0,
                travel_m: m(*travel_cm),
            },
            (
                Goal::Fill { count, .. },
                Some(PartSpec::Basket {
                    x_cm,
                    y_cm,
                    inner_width_cm,
                    inner_height_cm,
                }),
            ) => {
                let (x, y, w, h) = (m(*x_cm), m(*y_cm), m(*inner_width_cm), m(*inner_height_cm));
                let floor_top = y + rules.basket_wall_m;
                // `count` fits usize on every target we build for.
                #[allow(clippy::cast_possible_truncation)]
                Kind::Fill {
                    count: *count as usize,
                    inner_m: [x - w / 2.0, floor_top, x + w / 2.0, floor_top + h],
                }
            }
            (Goal::Pop { .. }, _) => Kind::Pop,
            _ => Kind::Ring,
        };
        Self {
            kind,
            target,
            via,
            via_done: 0,
        }
    }

    /// Records waypoints touched this step and reports whether the goal is met.
    pub(crate) fn observe(
        &mut self,
        world: &World,
        step: &StepInputs<'_>,
        rules: &Rules,
        events: &mut StepEvents,
    ) -> bool {
        while let Some(next) = self.via.get(self.via_done).copied() {
            if !step.contacts.iter().any(|(a, b)| *a == next || *b == next) {
                break;
            }
            events.push(EventKind::Via(next));
            self.via_done += 1;
        }
        if self.via_done < self.via.len() {
            return false;
        }
        let t = self.target;
        match self.kind {
            Kind::Ring => step
                .impulses
                .iter()
                .any(|(a, b, imp)| (*a == t || *b == t) && *imp >= rules.bell_min_impulse_ns),
            Kind::Press {
                rest_y_m,
                cap_half_h_m,
                travel_m,
            } => world
                .pose(t)
                .is_some_and(|p| rest_y_m - (p.y_m - cap_half_h_m) >= travel_m - PRESS_SLACK_M),
            Kind::Fill {
                count,
                inner_m: [x0, y0, x1, y1],
            } => {
                let resting = world
                    .dynamic_states()
                    .filter(|s| s.part != t && s.speed_m_s < rules.rest_speed_m_s)
                    .filter(|s| s.x_m > x0 && s.x_m < x1 && s.y_m > y0 && s.y_m < y1)
                    .count();
                resting >= count
            }
            Kind::Pop => step.popped.contains(&t),
        }
    }
}

/// What one step produced, as the tracker needs it.
pub(crate) struct StepInputs<'a> {
    pub contacts: &'a [(PartId, PartId)],
    pub impulses: &'a [(PartId, PartId, f32)],
    /// Balloons a pin popped this step. A balloon that floats out of the world is not popped.
    pub popped: &'a [PartId],
}
