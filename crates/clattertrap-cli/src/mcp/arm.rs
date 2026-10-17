//! The pilot arms: how many simulator runs a contestant gets before it submits, and the caps every
//! session shares.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Which arm a session plays under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    /// No simulator runs: check and look, then submit blind.
    Blueprint,
    /// Up to eight simulator runs before submitting.
    Workshop,
}

impl Arm {
    /// How many `run` calls the arm allows.
    #[must_use]
    pub const fn run_budget(self) -> u32 {
        match self {
            Self::Blueprint => 0,
            Self::Workshop => 8,
        }
    }

    /// The dollar cap on one session, the same for every arm and model.
    pub const MAX_BUDGET_USD: f64 = 6.0;

    /// The wall-clock cap on one session in seconds, the same for every arm and model.
    pub const WALL_CLOCK_S: u64 = 1200;

    /// The directory prefix of a run under this arm, as in `blueprint-01`.
    #[must_use]
    pub const fn dir_prefix(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Workshop => "workshop",
        }
    }
}

impl fmt::Display for Arm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.dir_prefix())
    }
}
