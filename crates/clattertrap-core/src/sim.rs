//! `Simulator::step` advances the world one fixed step and returns that step's events.
//! The rapier world is the only mutable state.

use crate::events::{Event, EventKind, PartId, StepEvents};
use crate::level::Level;
use crate::rules::Rules;
use crate::score::{Closing, End, Outcome, Recorder};
use crate::validate::ValidScene;
use crate::world::{BodyPose, World};

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
}

impl Simulator {
    /// A world at step 0 for this level and validated scene.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn new(level: &Level, scene: &ValidScene, rules: &Rules) -> Self {
        Self {
            world: World::build(level, scene, rules),
            rules: *rules,
            names: scene.names().to_vec(),
            step: 0,
            limit_steps: libm::ceilf(level.time_limit_s / rules.dt_s) as u32,
            parts_used: scene.scene_part_count(),
            events: StepEvents::new(),
            finished: None,
        }
    }

    /// Advances one fixed step and returns its events, sorted. Returns nothing once the run has ended.
    pub fn step(&mut self) -> &[Event] {
        self.events.clear();
        if self.finished.is_some() {
            return self.events.events();
        }
        let raw = self.world.step();
        for (a, b) in &raw.contacts {
            self.events.push(EventKind::contact(*a, *b));
        }
        self.step += 1;
        self.events.finish_step(self.step);
        if self.step >= self.limit_steps {
            self.finished = Some(End::TimeLimit);
        } else if self.world.all_asleep() {
            self.finished = Some(End::AllAsleep);
        }
        self.events.events()
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
    let end = loop {
        recorder.record(sim.step());
        if let Some(end) = sim.finished() {
            break end;
        }
    };
    let closing = Closing {
        end,
        steps: sim.step_index(),
        dt_s: rules.dt_s,
        parts_used: sim.parts_used,
        chain_length: 0,
        state_hash: sim.state_hash(),
        rules_version: rules.version,
    };
    let (outcome, events) = recorder.finish(closing);
    Run { outcome, events }
}
