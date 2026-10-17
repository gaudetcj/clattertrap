//! What a session writes to its run directory: one file per scene-taking call and the submission.

use super::arm::Arm;
use anyhow::{Context, Result};
use clattertrap_core::score::Outcome;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// One `check`, `look`, `run`, or `submit` call with the scene it was given and what came back.
#[derive(Debug, Serialize)]
pub struct Call<'a> {
    /// Position of the call in the session, from 1.
    pub seq: u32,
    /// The tool name.
    pub tool: &'static str,
    /// The scene as the contestant sent it.
    pub scene: &'a Value,
    /// Validation problems, when the rules rejected the scene.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<String>>,
    /// The run's outcome, when the scene was simulated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<&'a Outcome>,
    /// The machine name, for `submit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,
    /// The prediction, for `submit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<&'a str>,
}

/// The one submission of a session.
#[derive(Debug, Serialize)]
pub struct Submission<'a> {
    /// The level id.
    pub level: &'a str,
    /// The arm the session played under.
    pub arm: Arm,
    /// The machine name.
    pub name: &'a str,
    /// The one-sentence prediction.
    pub prediction: &'a str,
    /// The submitted scene.
    pub scene: &'a Value,
    /// Its outcome.
    pub outcome: &'a Outcome,
}

/// Writes a call as `calls/NNNN-<tool>.json` under `dir`.
///
/// # Errors
/// When the directory or the file cannot be written.
pub fn write_call(dir: &Path, call: &Call<'_>) -> Result<()> {
    let calls = dir.join("calls");
    std::fs::create_dir_all(&calls).with_context(|| format!("creating {}", calls.display()))?;
    let path = calls.join(format!("{:04}-{}.json", call.seq, call.tool));
    write_json(&path, call)
}

/// Writes the submission as `submission.json` under `dir`.
///
/// # Errors
/// When the directory or the file cannot be written.
pub fn write_submission(dir: &Path, submission: &Submission<'_>) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    write_json(&dir.join("submission.json"), submission)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}
