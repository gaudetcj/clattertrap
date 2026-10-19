//! What every provider is given for one session and what it hands back.

use crate::mcp::arm::Arm;
use crate::records::transcript::Record;
use anyhow::Result;
use std::future::Future;
use std::path::Path;
use std::time::Duration;

/// One session to run: the directory to work in, the MCP server to spawn, and the caps.
#[derive(Debug)]
pub struct Session<'a> {
    /// The run directory, also the session's working directory.
    pub dir: &'a Path,
    /// This binary, which serves the MCP tools.
    pub exe: &'a Path,
    /// The level id.
    pub level: &'a str,
    /// The arm.
    pub arm: Arm,
    /// The filled contestant prompt.
    pub prompt: &'a str,
    /// The wall clock for the whole session.
    pub wall_clock: Duration,
    /// The dollar cap.
    pub max_budget_usd: f64,
}

/// What one session left behind, the same shape for every provider.
#[derive(Debug, Default)]
pub struct Finished {
    /// The transcript records, in order.
    pub records: Vec<Record>,
    /// The model id the provider reported, when it did.
    pub model: Option<String>,
    /// Whether the MCP server connected.
    pub connected: bool,
    /// Whether the wall clock ran out.
    pub timed_out: bool,
    /// The contestant process's exit code, when there was a process and it ended on its own.
    pub exit_code: Option<i32>,
    /// The error text of a session that ended on a failure.
    pub error: Option<String>,
    /// The provider's raw output, kept beside the transcript for debugging.
    pub raw: String,
}

/// A way to run a contestant: Claude Code headless, or a chat endpoint.
pub trait Provider {
    /// Runs one session and returns what it left behind.
    ///
    /// # Errors
    /// When the session cannot be started or its output cannot be read.
    fn run(&self, session: &Session<'_>) -> impl Future<Output = Result<Finished>>;
}
