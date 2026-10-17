//! The pilot arms: how many simulator runs a contestant gets before it submits.

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
