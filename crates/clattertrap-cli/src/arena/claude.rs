//! The Claude Code headless provider: the command line for one session and running it.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// The MCP server name in the per-run config; tool names are `mcp__clattertrap__<tool>`.
pub const SERVER_NAME: &str = "clattertrap";

/// Whether the pilot may run this model: a Claude alias or a `claude-` model id.
#[must_use]
pub fn is_claude_model(model: &str) -> bool {
    matches!(model, "haiku" | "sonnet" | "opus" | "fable")
        || (model.starts_with("claude-") && model.len() > "claude-".len())
}

/// The arguments after `claude`: print mode on stdin, only the clattertrap MCP tools, structured
/// output, and the dollar cap. `--restricted` drops the built-in tools and ignores settings files.
#[must_use]
pub fn command_args(model: &str, mcp_config: &Path, max_budget_usd: f64) -> Vec<String> {
    [
        "-p",
        "--model",
        model,
        "--mcp-config",
        &mcp_config.display().to_string(),
        "--strict-mcp-config",
        "--restricted",
        "--tools",
        "",
        "--allowedTools",
        &format!("mcp__{SERVER_NAME}__*"),
        "--permission-mode",
        "dontAsk",
        "--permission-prompts",
        "none",
        "--output-format",
        "stream-json",
        "--verbose",
        "--no-session-persistence",
        "--max-budget-usd",
        &max_budget_usd.to_string(),
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// What one session left behind.
#[derive(Debug)]
pub struct Finished {
    /// The stream-json transcript from stdout.
    pub transcript: String,
    /// Everything on stderr.
    pub stderr: String,
    /// Whether the wall clock ran out and the process was killed.
    pub timed_out: bool,
    /// The exit code, when the process ended on its own.
    pub exit_code: Option<i32>,
}

/// Runs `claude` with `args` in `cwd`, feeding `prompt` on stdin, for at most `wall_clock`.
///
/// # Errors
/// When the process cannot be spawned or its output cannot be read.
pub async fn run_session(
    claude: &str,
    args: &[String],
    cwd: &Path,
    prompt: &str,
    wall_clock: Duration,
) -> Result<Finished> {
    let mut child = Command::new(claude)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("starting {claude}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .context("writing the prompt")?;
        drop(stdin);
    }
    let output = tokio::time::timeout(wall_clock, child.wait_with_output()).await;
    match output {
        Ok(output) => {
            let output = output.context("waiting for claude")?;
            Ok(Finished {
                transcript: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                timed_out: false,
                exit_code: output.status.code(),
            })
        }
        // The child is killed on drop; its partial output is lost with it.
        Err(_) => Ok(Finished {
            transcript: String::new(),
            stderr: String::new(),
            timed_out: true,
            exit_code: None,
        }),
    }
}
