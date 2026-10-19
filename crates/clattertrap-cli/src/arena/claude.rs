//! The Claude Code headless provider: the command line for one session, running it, and reading
//! its stream-json output.

use super::SERVER_NAME;
use super::provider::{Finished, Provider, Session};
use super::stream;
use anyhow::{Context, Result, ensure};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Claude Code in print mode, restricted to the clattertrap MCP tools.
#[derive(Debug)]
pub struct ClaudeCode {
    claude: String,
    model: String,
}

impl ClaudeCode {
    /// A provider running `claude` for `model`.
    ///
    /// # Errors
    /// When the model is not a Claude alias or id.
    pub fn new(claude: String, model: &str) -> Result<Self> {
        ensure!(
            is_claude_model(model),
            "Claude Code runs Claude models only; {model:?} is not one. Give --endpoint for others"
        );
        Ok(Self {
            claude,
            model: model.to_owned(),
        })
    }
}

impl Provider for ClaudeCode {
    async fn run(&self, s: &Session<'_>) -> Result<Finished> {
        let args = command_args(&self.model, &s.dir.join("mcp.json"), s.max_budget_usd);
        let output = run_session(&self.claude, &args, s.dir, s.prompt, s.wall_clock).await?;
        std::fs::write(s.dir.join("stderr.txt"), &output.stderr).context("writing stderr.txt")?;
        let parsed = stream::parse(&output.transcript);
        Ok(Finished {
            records: parsed.records,
            model: parsed.model,
            connected: parsed.connected,
            timed_out: output.timed_out,
            exit_code: output.exit_code,
            error: parsed.error,
            raw: output.transcript,
        })
    }
}

/// Whether Claude Code may run this model: a Claude alias or a `claude-` model id.
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

/// What one process left behind.
#[derive(Debug)]
pub struct Output {
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
) -> Result<Output> {
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
            Ok(Output {
                transcript: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                timed_out: false,
                exit_code: output.status.code(),
            })
        }
        // The child is killed on drop; its partial output is lost with it.
        Err(_) => Ok(Output {
            transcript: String::new(),
            stderr: String::new(),
            timed_out: true,
            exit_code: None,
        }),
    }
}
