//! Tracks a level's goal and its `via` waypoints as the run plays out. The waypoints and the goal
//! must be met by one chain: parts that have touched each other, directly or through other parts,
//! since the start. A part that moves on its own and never meets the chain cannot do its work.

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

/// Parts grouped into chains by contact, each chain with the number of waypoints it has touched in
/// order. A union-find over part ids; `parent[i] == i` marks a chain's root.
struct Chains {
    parent: Vec<u16>,
    progress: Vec<u8>,
}

impl Chains {
    fn new(parts: usize) -> Self {
        // validate caps the part count at u16::MAX.
        let parent = (0..parts)
            .map(|i| u16::try_from(i).unwrap_or(u16::MAX))
            .collect();
        Self {
            parent,
            progress: vec![0; parts],
        }
    }

    fn root(&mut self, part: PartId) -> usize {
        let mut i = usize::from(part.0);
        while usize::from(self.parent[i]) != i {
            let up = self.parent[usize::from(self.parent[i])];
            self.parent[i] = up;
            i = usize::from(up);
        }
        i
    }

    /// Merges two parts' chains; the merged chain keeps the further progress.
    fn join(&mut self, a: PartId, b: PartId) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            self.parent[rb] = u16::try_from(ra).unwrap_or(u16::MAX);
            self.progress[ra] = self.progress[ra].max(self.progress[rb]);
        }
    }

    fn progress(&mut self, part: PartId) -> usize {
        let r = self.root(part);
        usize::from(self.progress[r])
    }

    fn advance(&mut self, part: PartId) {
        let r = self.root(part);
        self.progress[r] = self.progress[r].saturating_add(1);
    }
}

/// The goal, resolved to part ids, with every chain's waypoint progress.
pub(crate) struct Tracker {
    kind: Kind,
    target: PartId,
    via: Vec<PartId>,
    actors: Vec<PartId>,
    floor: PartId,
    chains: Chains,
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
        // The level's moving fixtures other than the goal object; the goal must be their chain's work.
        let actors = level
            .fixed
            .iter()
            .filter(|p| p.spec.is_dynamic() && p.id != level.goal.target())
            .filter_map(|p| scene.part_id(&p.id))
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
        let floor = scene.floor_id();
        let mut chains = Chains::new(usize::from(floor.0) + 1);
        // A rope moves its two parts together, so they start as one chain.
        for p in scene.placed() {
            if let PartSpec::Rope { a, b, .. } = &p.spec
                && let (Some(x), Some(y)) = (scene.part_id(a), scene.part_id(b))
            {
                chains.join(x, y);
            }
        }
        Self {
            kind,
            target,
            via,
            actors,
            floor,
            chains,
        }
    }

    /// Records waypoints touched this step and reports whether the goal is met. A touch is a contact
    /// with anything but the floor while one of the pair is moving, so resting contacts never count,
    /// and it advances the toucher's chain only when the waypoint is that chain's next one.
    pub(crate) fn observe(
        &mut self,
        world: &World,
        step: &StepInputs<'_>,
        rules: &Rules,
        events: &mut StepEvents,
    ) -> bool {
        for &(a, b) in step.contacts {
            if world.is_dynamic(a) && world.is_dynamic(b) {
                self.chains.join(a, b);
            }
        }
        let mut advanced = true;
        while advanced {
            advanced = false;
            for &(a, b) in step.contacts {
                if a == self.floor
                    || b == self.floor
                    || !(moving(world, a, rules) || moving(world, b, rules))
                {
                    continue;
                }
                for (waypoint, other) in [(a, b), (b, a)] {
                    // Only a dynamic toucher belongs to a chain; a beam the weight brushes carries nothing.
                    if world.is_dynamic(other)
                        && self.via.get(self.chains.progress(other)) == Some(&waypoint)
                    {
                        self.chains.advance(other);
                        events.push(EventKind::Via(waypoint));
                        advanced = true;
                    }
                }
            }
        }
        let t = self.target;
        match self.kind {
            Kind::Ring => step.impulses.iter().any(|&(a, b, imp)| {
                imp >= rules.bell_min_impulse_ns
                    && ((a == t && self.rings(world, b)) || (b == t && self.rings(world, a)))
            }),
            Kind::Press {
                rest_y_m,
                cap_half_h_m,
                travel_m,
            } => {
                self.complete(t)
                    && self.with_actor(t)
                    && world.pose(t).is_some_and(|p| {
                        rest_y_m - (p.y_m - cap_half_h_m) >= travel_m - PRESS_SLACK_M
                    })
            }
            Kind::Fill {
                count,
                inner_m: [x0, y0, x1, y1],
            } => {
                let mut resting = 0;
                for s in world.dynamic_states() {
                    let inside = s.x_m > x0 && s.x_m < x1 && s.y_m > y0 && s.y_m < y1;
                    if s.part != t
                        && s.speed_m_s < rules.rest_speed_m_s
                        && inside
                        && self.complete(s.part)
                        && self.with_actor(s.part)
                    {
                        resting += 1;
                    }
                }
                resting >= count
            }
            Kind::Pop => step.popped.contains(&t) && self.complete(t) && self.with_actor(t),
        }
    }

    /// Whether a part can ring the bell: a dynamic part whose chain is complete. A static part never
    /// does, since the push the solver puts on a beam resting against the bell is not a blow.
    fn rings(&mut self, world: &World, part: PartId) -> bool {
        world.is_dynamic(part) && self.complete(part) && self.with_actor(part)
    }

    /// Whether a part's chain includes one of the level's moving fixtures, or the level has none.
    fn with_actor(&mut self, part: PartId) -> bool {
        let root = self.chains.root(part);
        self.actors.is_empty() || self.actors.iter().any(|&a| self.chains.root(a) == root)
    }

    /// Whether a part's chain has touched every waypoint.
    fn complete(&mut self, part: PartId) -> bool {
        self.chains.progress(part) >= self.via.len()
    }
}

/// What one step produced, as the tracker needs it.
pub(crate) struct StepInputs<'a> {
    pub contacts: &'a [(PartId, PartId)],
    pub impulses: &'a [(PartId, PartId, f32)],
    /// Balloons a pin popped this step. A balloon that floats out of the world is not popped.
    pub popped: &'a [PartId],
}

/// Whether a part moves faster than the chain threshold; fixed and removed parts never do.
fn moving(world: &World, part: PartId, rules: &Rules) -> bool {
    let s = rules.chain_move_speed_m_s;
    world
        .velocity(part)
        .is_some_and(|[vx, vy]| vx * vx + vy * vy > s * s)
}
