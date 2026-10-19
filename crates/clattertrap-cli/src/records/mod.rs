//! The records a session leaves on disk and every later command reads: calls, the submission, and
//! the runner's summary. The shapes are documented in docs/format.md.

#[cfg(test)]
mod tests;

pub mod transcript;

use crate::mcp::arm::Arm;
use anyhow::{Context, Result};
use clattertrap_core::score::Outcome;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};

/// The scene-taking tools, as `calls/NNNN-<tool>.json` names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    /// Validate a scene.
    Check,
    /// Draw a scene at the start.
    Look,
    /// Simulate a scene.
    Run,
    /// Submit the machine.
    Submit,
}

impl Tool {
    /// The tool name as the MCP server lists it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Look => "look",
            Self::Run => "run",
            Self::Submit => "submit",
        }
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One `check`, `look`, `run`, or `submit` call with the scene it was given and what came back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    /// Position of the call in the session, from 1.
    pub seq: u32,
    /// The tool.
    pub tool: Tool,
    /// The scene as the contestant sent it.
    pub scene: Value,
    /// Validation problems, when the rules rejected the scene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<String>>,
    /// The run's outcome, when the scene was simulated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,
    /// The machine name, for `submit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The prediction, for `submit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prediction: Option<String>,
}

impl Call {
    /// Whether the rules accepted the scene.
    #[must_use]
    pub const fn accepted(&self) -> bool {
        self.problems.is_none()
    }
}

/// The one submission of a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Submission {
    /// The level id.
    pub level: String,
    /// The arm the session played under.
    pub arm: Arm,
    /// The machine name.
    pub name: String,
    /// The one-sentence prediction.
    pub prediction: String,
    /// The submitted scene.
    pub scene: Value,
    /// Its outcome.
    pub outcome: Outcome,
}

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

    /// The status as the records spell it.
    #[must_use]
    pub fn name(self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(ToOwned::to_owned))
            .unwrap_or_default()
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
    /// The runner's cost estimate, when the session reported one.
    pub cost_usd: Option<f64>,
    /// Turns, when the session reported them.
    pub turns: Option<u32>,
    /// Tool calls the contestant made.
    pub tool_calls: u32,
    /// Tool names outside the clattertrap MCP tools, which flag the run.
    pub foreign_tools: Vec<String>,
    /// The `subtype` of the final result record.
    pub result_subtype: Option<String>,
    /// The exit code of the contestant process, when it ended on its own.
    pub exit_code: Option<i32>,
    /// The error text, for an aborted run.
    pub error: Option<String>,
}

/// The scene JSON text behind a recorded scene: a string value as is, anything else serialized.
#[must_use]
pub fn scene_json(scene: &Value) -> String {
    match scene {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Reads and parses one JSON file.
///
/// # Errors
/// When the file cannot be read or does not parse as `T`.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Writes one value as pretty JSON with a trailing newline.
///
/// # Errors
/// When the file cannot be written.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// `dir/calls/NNNN-<tool>.json`.
#[must_use]
pub fn call_path(dir: &Path, seq: u32, tool: Tool) -> PathBuf {
    dir.join("calls").join(format!("{seq:04}-{tool}.json"))
}

/// Writes a call under `dir`.
///
/// # Errors
/// When the directory or the file cannot be written.
pub fn write_call(dir: &Path, call: &Call) -> Result<()> {
    let calls = dir.join("calls");
    std::fs::create_dir_all(&calls).with_context(|| format!("creating {}", calls.display()))?;
    write_json(&call_path(dir, call.seq, call.tool), call)
}

/// Writes `submission.json` under `dir`.
///
/// # Errors
/// When the directory or the file cannot be written.
pub fn write_submission(dir: &Path, submission: &Submission) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    write_json(&dir.join("submission.json"), submission)
}

/// Writes `summary.json` under `dir`.
///
/// # Errors
/// When the file cannot be written.
pub fn write_summary(dir: &Path, summary: &Summary) -> Result<()> {
    write_json(&dir.join("summary.json"), summary)
}

/// Every `calls/*.json` under `dir` in name order, each parsed or kept as its own error; none when
/// there is no `calls/` directory.
///
/// # Errors
/// When `calls/` exists but cannot be listed.
pub fn read_calls(dir: &Path) -> Result<Vec<(PathBuf, Result<Call>)>> {
    let calls = dir.join("calls");
    let Ok(entries) = std::fs::read_dir(&calls) else {
        return Ok(vec![]);
    };
    let mut paths: Vec<PathBuf> = entries
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<_>>()
        .with_context(|| format!("listing {}", calls.display()))?;
    paths.retain(|p| p.extension().is_some_and(|e| e == "json"));
    paths.sort();
    Ok(paths
        .into_iter()
        .map(|path| {
            let call = read_json(&path);
            (path, call)
        })
        .collect())
}

/// Every directory three levels under `dir` that holds a `summary.json`, sorted: the pilot layout
/// `<model>/<level>/<arm>-NN` and the gallery layout `<level>/<model>/<arm>-NN` alike. None when
/// `dir` is missing.
#[must_use]
pub fn run_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for a in subdirs(dir) {
        for b in subdirs(&a) {
            for c in subdirs(&b) {
                if c.join("summary.json").is_file() {
                    out.push(c);
                }
            }
        }
    }
    out
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}
