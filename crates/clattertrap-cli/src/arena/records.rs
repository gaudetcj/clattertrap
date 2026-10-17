//! What the runner writes for every run: its directory and `summary.json`.

use crate::mcp::arm::Arm;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The contestant submitted a machine.
    Submitted,
    /// The contestant ended its session without submitting.
    NoSubmission,
    /// The session hit the dollar cap before submitting.
    OverBudget,
    /// The session hit the wall-clock cap before submitting.
    TimedOut,
    /// The session ended on an error, a usage limit, or a server that never connected.
    Aborted,
    /// The session called a tool outside the clattertrap MCP tools.
    Flagged,
}

impl Status {
    /// Whether the run counts as a result: a submission or the contestant's own failure to make one.
    #[must_use]
    pub const fn counts(self) -> bool {
        matches!(
            self,
            Self::Submitted | Self::NoSubmission | Self::OverBudget | Self::TimedOut
        )
    }
}

/// One run's `summary.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    /// The model id as the runner was given it.
    pub model: String,
    /// The level id.
    pub level: String,
    /// The arm.
    pub arm: Arm,
    /// How the run ended.
    pub status: Status,
    /// When the session started, in seconds since the Unix epoch.
    pub started_unix_s: u64,
    /// Wall-clock seconds from spawn to exit.
    pub duration_s: f64,
    /// Claude Code's cost estimate, when the session reported one.
    pub cost_usd: Option<f64>,
    /// Turns, when the session reported them.
    pub turns: Option<u32>,
    /// Tool calls the contestant made.
    pub tool_calls: u32,
    /// Tool names outside the clattertrap MCP tools, which flag the run.
    pub foreign_tools: Vec<String>,
    /// The `subtype` of the final result record.
    pub result_subtype: Option<String>,
    /// The exit code of the claude process, when it ended on its own.
    pub exit_code: Option<i32>,
    /// The error text, for an aborted run.
    pub error: Option<String>,
}

/// The next unused run directory: `<out>/<model>/<level>/<arm>-NN`, two digits from 01.
///
/// # Errors
/// When every index up to 99 is taken.
pub fn next_run_dir(out: &Path, model: &str, level: &str, arm: Arm) -> Result<PathBuf> {
    let parent = out.join(model).join(level);
    (1..=99)
        .map(|n| parent.join(format!("{}-{n:02}", arm.dir_prefix())))
        .find(|dir| !dir.exists())
        .with_context(|| format!("{} has 99 {} runs already", parent.display(), arm))
}

/// Writes `summary.json` into `dir`.
///
/// # Errors
/// When the file cannot be written.
pub fn write_summary(dir: &Path, summary: &Summary) -> Result<()> {
    let mut text = serde_json::to_string_pretty(summary)?;
    text.push('\n');
    let path = dir.join("summary.json");
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}
