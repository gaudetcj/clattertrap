//! wasm-bindgen wrapper over `clattertrap-core` for the browser viewer. No logic lives here: `Replay`
//! parses and validates the JSON it is given, steps the simulator, and hands out what core reports.
#![forbid(unsafe_code)]

use clattertrap_core::events::{Event, EventKind, PartId};
use clattertrap_core::level::{Goal, Level, Zone};
use clattertrap_core::parts::Visual;
use clattertrap_core::rules::{RULES_VERSION, Rules};
use clattertrap_core::scene::Scene;
use clattertrap_core::score::Recorder;
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::validate;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

/// The rules version this build simulates under.
#[wasm_bindgen]
#[must_use]
pub fn rules_version() -> u32 {
    RULES_VERSION
}

/// The fixed time step in seconds, so the page never hard-codes it.
#[wasm_bindgen]
#[must_use]
pub fn dt_s() -> f32 {
    Rules::V1.dt_s
}

/// One run of a scene on a level, stepped from the page.
#[wasm_bindgen]
pub struct Replay {
    level: Level,
    sim: Simulator,
    recorder: Option<Recorder>,
    outcome: Option<String>,
}

/// What the page needs once, before the first step.
#[derive(Serialize)]
struct Setup<'a> {
    width_cm: f32,
    height_cm: f32,
    time_limit_s: f32,
    zones: &'a [Zone],
    goal: &'a Goal,
    names: &'a [String],
    slots: Vec<SlotInfo>,
}

/// A body slot's part and drawing description. Poses come per frame.
#[derive(Serialize)]
struct SlotInfo {
    part: u16,
    visual: Visual,
}

#[wasm_bindgen]
impl Replay {
    /// Parses and validates a level and a scene. The error lists every validation problem, one per line.
    ///
    /// # Errors
    /// When either JSON does not parse or the scene fails validation.
    #[wasm_bindgen(constructor)]
    pub fn new(level_json: &str, scene_json: &str) -> Result<Replay, String> {
        let level: Level = serde_json::from_str(level_json).map_err(|e| format!("level: {e}"))?;
        let scene: Scene = serde_json::from_str(scene_json).map_err(|e| format!("scene: {e}"))?;
        let valid = validate(&level, &scene, &Rules::V1).map_err(|problems| {
            problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let sim = Simulator::new(&level, &valid, &Rules::V1);
        Ok(Self {
            level,
            sim,
            recorder: Some(Recorder::new()),
            outcome: None,
        })
    }

    /// World size, zones, goal, part names, and one entry per body slot, as JSON.
    ///
    /// # Errors
    /// When serialization fails, which the types rule out.
    pub fn setup(&self) -> Result<String, String> {
        let setup = Setup {
            width_cm: self.level.world.width_cm,
            height_cm: self.level.world.height_cm,
            time_limit_s: self.level.time_limit_s,
            zones: &self.level.zones,
            goal: &self.level.goal,
            names: self.sim.names(),
            slots: self
                .sim
                .slots()
                .map(|s| SlotInfo {
                    part: s.part.0,
                    visual: s.visual,
                })
                .collect(),
        };
        serde_json::to_string(&setup).map_err(|e| e.to_string())
    }

    /// Advances one step and returns its events as `(kind, a, b)` triples.
    ///
    /// Kinds: 0 contact `a`,`b`; 1 via `a`; 2 goal met; 3 at rest `a`; 4 left world `a`.
    pub fn step(&mut self) -> Vec<u16> {
        let events = self.sim.step();
        if let Some(r) = self.recorder.as_mut() {
            r.record(events);
        }
        events.iter().flat_map(|e| encode(*e)).collect()
    }

    /// Four numbers per slot: x in metres, y in metres, angle in radians, and 1 while the body exists.
    pub fn frame(&self) -> Vec<f32> {
        self.sim
            .slots()
            .flat_map(|s| match s.pose {
                Some(p) => [p.x_m, p.y_m, p.angle_rad, 1.0],
                None => [0.0; 4],
            })
            .collect()
    }

    /// Linear velocity of a part in m/s as `[x, y]`, or empty once its body is gone.
    pub fn velocity(&self, part: u16) -> Vec<f32> {
        self.sim
            .velocity(PartId(part))
            .map_or_else(Vec::new, |v| v.to_vec())
    }

    /// Steps taken so far.
    #[must_use]
    pub fn step_index(&self) -> u32 {
        self.sim.step_index()
    }

    /// The outcome as JSON once the run has ended, or `None` while it is running.
    ///
    /// # Errors
    /// When serialization fails, which the types rule out.
    pub fn outcome(&mut self) -> Result<Option<String>, String> {
        if self.outcome.is_none()
            && let Some(closing) = self.sim.closing()
            && let Some(recorder) = self.recorder.take()
        {
            let (outcome, _) = recorder.finish(closing);
            self.outcome = Some(serde_json::to_string(&outcome).map_err(|e| e.to_string())?);
        }
        Ok(self.outcome.clone())
    }
}

fn encode(e: Event) -> [u16; 3] {
    match e.kind {
        EventKind::Contact { a, b } => [0, a.0, b.0],
        EventKind::Via(p) => [1, p.0, 0],
        EventKind::GoalMet => [2, 0, 0],
        EventKind::AtRest(p) => [3, p.0, 0],
        EventKind::LeftWorld(p) => [4, p.0, 0],
    }
}
