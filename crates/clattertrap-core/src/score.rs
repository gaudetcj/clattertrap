//! Folds a run's events into an `Outcome`.

use crate::events::Event;
use crate::hash::Fnv64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Why a run stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum End {
    /// The goal was met.
    GoalMet,
    /// The level's time limit passed.
    TimeLimit,
    /// Every body fell asleep.
    AllAsleep,
}

/// Hashes serialize as 16 hex digits because JSON numbers cannot hold a u64 exactly.
#[allow(clippy::trivially_copy_pass_by_ref)] // serde's serialize_with signature
fn hex_u64<S: Serializer>(value: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{value:016x}"))
}

/// Parses the 16 hex digits `hex_u64` writes.
fn parse_hex_u64<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let text = String::deserialize(d)?;
    u64::from_str_radix(&text, 16).map_err(serde::de::Error::custom)
}

/// The result of one run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    /// Whether the goal was met.
    pub goal_met: bool,
    /// Why the run stopped.
    pub end: End,
    /// Simulated time at the end, `steps * dt_s`.
    pub time_s: f32,
    /// Steps simulated.
    pub steps: u32,
    /// Parts the contestant placed.
    pub parts_used: u32,
    /// Distinct parts set moving before the goal, or before the end if the goal was missed.
    pub chain_length: u32,
    /// FNV-1a 64 over the event log.
    #[serde(serialize_with = "hex_u64", deserialize_with = "parse_hex_u64")]
    pub event_hash: u64,
    /// FNV-1a 64 over the final body states.
    #[serde(serialize_with = "hex_u64", deserialize_with = "parse_hex_u64")]
    pub state_hash: u64,
    /// Rules the run used.
    pub rules_version: u32,
}

/// Collects events and hashes them as they arrive.
#[derive(Debug, Default)]
pub struct Recorder {
    events: Vec<Event>,
    hash: Fnv64,
}

/// Everything `Recorder::finish` needs beyond the events.
#[derive(Debug, Clone, Copy)]
pub struct Closing {
    /// Why the run stopped.
    pub end: End,
    /// Steps simulated.
    pub steps: u32,
    /// Fixed time step.
    pub dt_s: f32,
    /// Parts the contestant placed.
    pub parts_used: u32,
    /// Chain length at the end.
    pub chain_length: u32,
    /// Final state hash from the world.
    pub state_hash: u64,
    /// Rules version.
    pub rules_version: u32,
}

impl Recorder {
    /// Empty recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one step's events.
    pub fn record(&mut self, events: &[Event]) {
        for e in events {
            e.write_to(&mut self.hash);
        }
        self.events.extend_from_slice(events);
    }

    /// Produces the outcome and hands back the event log.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn finish(self, closing: Closing) -> (Outcome, Vec<Event>) {
        let outcome = Outcome {
            goal_met: closing.end == End::GoalMet,
            end: closing.end,
            time_s: closing.steps as f32 * closing.dt_s,
            steps: closing.steps,
            parts_used: closing.parts_used,
            chain_length: closing.chain_length,
            event_hash: self.hash.finish(),
            state_hash: closing.state_hash,
            rules_version: closing.rules_version,
        };
        (outcome, self.events)
    }
}
