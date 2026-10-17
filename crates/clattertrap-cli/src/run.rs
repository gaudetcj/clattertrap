//! Running a scene to its end, keeping the simulator for the final frame, and the event trace lines.

use clattertrap_core::events::{Event, EventKind, PartId};
use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::score::{Outcome, Recorder};
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::ValidScene;

/// Runs to the end, as core's `run` does, and also returns the simulator in its final state.
pub fn run_to_end(level: &Level, valid: &ValidScene) -> (Outcome, Vec<Event>, Simulator) {
    let mut sim = Simulator::new(level, valid, &Rules::V1);
    let mut recorder = Recorder::new();
    let closing = loop {
        recorder.record(sim.step());
        if let Some(closing) = sim.closing() {
            break closing;
        }
    };
    let (outcome, events) = recorder.finish(closing);
    (outcome, events, sim)
}

/// One trace line: the time in seconds, the event kind, and the parts involved by name.
#[allow(clippy::cast_precision_loss)]
pub fn trace_line(event: Event, names: &[String], dt_s: f32) -> String {
    let name = |p: PartId| names.get(usize::from(p.0)).map_or("?", String::as_str);
    let what = match event.kind {
        EventKind::Contact { a, b } => format!("contact {} {}", name(a), name(b)),
        EventKind::Via(p) => format!("via {}", name(p)),
        EventKind::GoalMet => "goal_met".to_owned(),
        EventKind::AtRest(p) => format!("at_rest {}", name(p)),
        EventKind::LeftWorld(p) => format!("left_world {}", name(p)),
    };
    format!("{:7.3} {what}", event.step as f32 * dt_s)
}
