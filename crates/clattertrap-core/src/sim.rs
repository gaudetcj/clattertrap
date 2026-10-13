//! `Simulator::step` advances the world one fixed step and returns that step's events.
//! The rapier world is the only mutable state.

use crate::events::{Event, EventKind, PartId, StepEvents};
use crate::goal::{StepInputs, Tracker};
use crate::level::Level;
use crate::parts::{m, spring_pad};
use crate::rules::Rules;
use crate::scene::PartSpec;
use crate::score::{Closing, End, Outcome, Recorder};
use crate::validate::ValidScene;
use crate::world::{BodyPose, Slot, World};

/// One run in progress.
pub struct Simulator {
    world: World,
    rules: Rules,
    names: Vec<String>,
    step: u32,
    limit_steps: u32,
    parts_used: u32,
    events: StepEvents,
    finished: Option<End>,
    reactive: Reactive,
    goal: Tracker,
    /// World rectangle plus the leave margin: min x, min y, max x, max y.
    bounds: [f32; 4],
    contacts: Vec<(PartId, PartId)>,
    impulses: Vec<(PartId, PartId, f32)>,
    leaving: Vec<PartId>,
    popped: Vec<PartId>,
    asleep: Vec<bool>,
    moved: Vec<bool>,
}

impl Simulator {
    /// A world at step 0 for this level and validated scene.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn new(level: &Level, scene: &ValidScene, rules: &Rules) -> Self {
        let n = scene.names().len();
        let margin = rules.leave_margin_m;
        Self {
            world: World::build(level, scene, rules),
            rules: *rules,
            names: scene.names().to_vec(),
            step: 0,
            limit_steps: libm::ceilf(level.time_limit_s / rules.dt_s) as u32,
            parts_used: scene.scene_part_count(),
            events: StepEvents::new(),
            finished: None,
            reactive: Reactive::from_scene(scene),
            goal: Tracker::new(level, scene, rules),
            bounds: [
                -margin,
                -margin,
                m(level.world.width_cm) + margin,
                m(level.world.height_cm) + margin,
            ],
            contacts: Vec::with_capacity(64),
            impulses: Vec::with_capacity(16),
            leaving: Vec::with_capacity(4),
            popped: Vec::with_capacity(2),
            asleep: vec![false; n],
            moved: vec![false; n],
        }
    }

    /// Advances one fixed step and returns its events, sorted. Returns nothing once the run has ended.
    pub fn step(&mut self) -> &[Event] {
        self.events.clear();
        if self.finished.is_some() {
            return self.events.events();
        }
        let raw = self.world.step();
        self.contacts.clear();
        self.contacts.extend_from_slice(&raw.contacts);
        self.impulses.clear();
        self.impulses.extend_from_slice(&raw.impulses);
        self.popped.clear();
        for (a, b) in &self.contacts {
            self.events.push(EventKind::contact(*a, *b));
        }
        self.react();
        self.sweep();
        let inputs = StepInputs {
            contacts: &self.contacts,
            impulses: &self.impulses,
            popped: &self.popped,
        };
        if self
            .goal
            .observe(&self.world, &inputs, &self.rules, &mut self.events)
        {
            self.events.push(EventKind::GoalMet);
            self.finished = Some(End::GoalMet);
        }
        self.step += 1;
        self.events.finish_step(self.step);
        if self.finished.is_none() {
            if self.step >= self.limit_steps {
                self.finished = Some(End::TimeLimit);
            } else if self.world.all_asleep() {
                self.finished = Some(End::AllAsleep);
            }
        }
        self.events.events()
    }

    /// Spring pads launch what touches them; pins pop balloons.
    fn react(&mut self) {
        for i in 0..self.contacts.len() {
            let (a, b) = self.contacts[i];
            for (x, y) in [(a, b), (b, a)] {
                if let Some((_, n)) = self.reactive.pads.iter().find(|(p, _)| *p == x)
                    && self.world.is_dynamic(y)
                {
                    let k = self.rules.spring_impulse_ns;
                    self.world.apply_impulse(y, [n[0] * k, n[1] * k]);
                }
                if self.reactive.pins.contains(&x) && self.reactive.balloons.contains(&y) {
                    self.world.remove_part(y);
                    self.popped.push(y);
                }
            }
        }
    }

    /// Removes bodies that left the world, notes which parts moved, and reports parts that fell asleep.
    fn sweep(&mut self) {
        self.leaving.clear();
        let [x0, y0, x1, y1] = self.bounds;
        for s in self.world.dynamic_states() {
            let i = usize::from(s.part.0);
            if s.x_m < x0 || s.x_m > x1 || s.y_m < y0 || s.y_m > y1 {
                self.leaving.push(s.part);
                continue;
            }
            if s.speed_m_s > self.rules.chain_move_speed_m_s {
                self.moved[i] = true;
            }
            if s.asleep && !self.asleep[i] {
                self.events.push(EventKind::AtRest(s.part));
            }
            self.asleep[i] = s.asleep;
        }
        for i in 0..self.leaving.len() {
            let part = self.leaving[i];
            self.events.push(EventKind::LeftWorld(part));
            self.world.remove_part(part);
        }
    }

    /// Steps taken so far.
    #[must_use]
    pub const fn step_index(&self) -> u32 {
        self.step
    }

    /// Simulated time, `step * dt_s`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn time_s(&self) -> f32 {
        self.step as f32 * self.rules.dt_s
    }

    /// Why the run ended, once it has.
    #[must_use]
    pub const fn finished(&self) -> Option<End> {
        self.finished
    }

    /// Distinct parts set moving so far.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn chain_length(&self) -> u32 {
        self.moved.iter().filter(|m| **m).count() as u32
    }

    /// The id of a named part.
    #[must_use]
    pub fn part_id(&self, name: &str) -> Option<PartId> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(PartId)
    }

    /// Part names by id.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Every body's pose.
    pub fn poses(&self) -> impl Iterator<Item = BodyPose> + '_ {
        self.world.poses()
    }

    /// Every body slot in insertion order, removed bodies included with no pose.
    pub fn slots(&self) -> impl Iterator<Item = Slot> + '_ {
        self.world.slots()
    }

    /// What `Recorder::finish` needs, once the run has ended.
    #[must_use]
    pub fn closing(&self) -> Option<Closing> {
        self.finished.map(|end| Closing {
            end,
            steps: self.step,
            dt_s: self.rules.dt_s,
            parts_used: self.parts_used,
            chain_length: self.chain_length(),
            state_hash: self.state_hash(),
            rules_version: self.rules.version,
        })
    }

    /// Pose of a part's body, if it still exists.
    #[must_use]
    pub fn pose(&self, part: PartId) -> Option<BodyPose> {
        self.world.pose(part)
    }

    /// Linear velocity of a part's body in m/s.
    #[must_use]
    pub fn velocity(&self, part: PartId) -> Option<[f32; 2]> {
        self.world.velocity(part)
    }

    /// Angular velocity of a part's body in rad/s.
    #[must_use]
    pub fn angular_velocity(&self, part: PartId) -> Option<f32> {
        self.world.angular_velocity(part)
    }

    /// Hash of the current body states.
    #[must_use]
    pub fn state_hash(&self) -> u64 {
        self.world.state_hash()
    }
}

/// Parts the simulator reacts to after each step, by id.
struct Reactive {
    /// Spring pads with their launch normals.
    pads: Vec<(PartId, [f32; 2])>,
    pins: Vec<PartId>,
    balloons: Vec<PartId>,
}

impl Reactive {
    #[allow(clippy::cast_possible_truncation)]
    fn from_scene(scene: &ValidScene) -> Self {
        let mut r = Self {
            pads: Vec::new(),
            pins: Vec::new(),
            balloons: Vec::new(),
        };
        for (i, placed) in scene.placed().iter().enumerate() {
            // validate caps the part count at u16::MAX.
            let id = PartId(i as u16);
            match placed.spec {
                PartSpec::SpringPad { angle_deg, .. } => {
                    r.pads.push((id, spring_pad::normal(angle_deg)));
                }
                PartSpec::Pin { .. } => r.pins.push(id),
                PartSpec::Balloon { .. } => r.balloons.push(id),
                _ => {}
            }
        }
        r
    }
}

/// A finished run: its outcome and full event log.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// Scored result.
    pub outcome: Outcome,
    /// Every event in step order.
    pub events: Vec<Event>,
}

/// Runs a scene to its end.
#[must_use]
pub fn run(level: &Level, scene: &ValidScene, rules: &Rules) -> Run {
    let mut sim = Simulator::new(level, scene, rules);
    let mut recorder = Recorder::new();
    let closing = loop {
        recorder.record(sim.step());
        if let Some(closing) = sim.closing() {
            break closing;
        }
    };
    let (outcome, events) = recorder.finish(closing);
    Run { outcome, events }
}
