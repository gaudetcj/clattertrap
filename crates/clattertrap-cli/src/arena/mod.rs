//! The arena runner: drives a contestant through the MCP tools and saves every run. One provider
//! so far, Claude Code headless.

mod claude;
mod records;
mod transcript;

#[cfg(test)]
mod tests;

use crate::catalog;
use crate::mcp::arm::Arm;
use anyhow::{Context, Result, ensure};
use claude::{SERVER_NAME, command_args, is_claude_model, run_session};
use records::{Summary, next_run_dir, write_summary};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use transcript::{Transcript, status};

/// The contestant prompt every model gets, with `{level}` and `{run_budget}` to fill.
const PROMPT: &str = include_str!("../../../../pilot/prompt.txt");

/// What one `arena` invocation runs.
#[derive(Debug)]
pub struct Options {
    /// The model, a Claude alias or id.
    pub model: String,
    /// Sessions per level.
    pub runs: u32,
    /// The arm.
    pub arm: Arm,
    /// One level instead of all of them.
    pub level: Option<String>,
    /// Where run directories go.
    pub out: PathBuf,
    /// The `claude` executable.
    pub claude: String,
}

/// `out` as an absolute path, so the per-run MCP config and the server's `--out` stay valid from any
/// working directory: the session runs with its run directory as cwd.
///
/// # Errors
/// When the current directory cannot be read.
pub fn absolute_out(out: &Path) -> Result<PathBuf> {
    std::path::absolute(out).with_context(|| format!("resolving {}", out.display()))
}

/// The prompt for one level under one arm.
#[must_use]
pub fn fill_prompt(level: &str, run_budget: u32) -> String {
    PROMPT
        .replace("{level}", level)
        .replace("{run_budget}", &run_budget.to_string())
}

/// The per-run MCP config: this binary serving `level` under `arm`, recording into `dir`.
#[must_use]
pub fn per_run_config(exe: &Path, level: &str, arm: Arm, dir: &Path) -> Value {
    json!({
        "mcpServers": {
            SERVER_NAME: {
                "command": exe.display().to_string(),
                "args": ["mcp", "--level", level, "--arm", arm.to_string(), "--out", dir.display().to_string()],
            }
        }
    })
}

/// Runs every level (or the one named) `runs` times, printing one line per run and, at the end, the
/// runs to do again.
///
/// # Errors
/// When the model is not a Claude model, a level is unknown, or a run cannot be started or saved.
pub fn run(opts: &Options) -> Result<()> {
    ensure!(
        is_claude_model(&opts.model),
        "the pilot runs Claude models only; {:?} is not one",
        opts.model
    );
    let levels = match &opts.level {
        Some(id) => vec![catalog::level(catalog::find(id)?)?],
        None => catalog::levels()?,
    };
    let out = absolute_out(&opts.out)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("starting the runtime")?;
    let mut again = Vec::new();
    for level in &levels {
        for _ in 0..opts.runs {
            let (dir, summary) = runtime.block_on(one_run(opts, &out, &level.id))?;
            println!("{}", run_line(&dir, &summary));
            if !summary.status.counts() {
                again.push(dir);
            }
        }
    }
    if !again.is_empty() {
        println!("to run again, with --level:");
        for dir in &again {
            println!("  {}", dir.display());
        }
    }
    Ok(())
}

async fn one_run(opts: &Options, out: &Path, level: &str) -> Result<(PathBuf, Summary)> {
    let dir = next_run_dir(out, &opts.model, level, opts.arm)?;
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let exe = std::env::current_exe().context("finding this executable")?;
    let config_path = dir.join("mcp.json");
    std::fs::write(
        &config_path,
        serde_json::to_string_pretty(&per_run_config(&exe, level, opts.arm, &dir))?,
    )
    .with_context(|| format!("writing {}", config_path.display()))?;
    let args = command_args(&opts.model, &config_path, Arm::MAX_BUDGET_USD);
    let prompt = fill_prompt(level, opts.arm.run_budget());
    let started_unix_s = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let clock = Instant::now();
    let finished = run_session(
        &opts.claude,
        &args,
        &dir,
        &prompt,
        Duration::from_secs(Arm::WALL_CLOCK_S),
    )
    .await?;
    let duration_s = clock.elapsed().as_secs_f64();
    std::fs::write(dir.join("transcript.jsonl"), &finished.transcript)?;
    std::fs::write(dir.join("stderr.txt"), &finished.stderr)?;
    let transcript = Transcript::parse(&finished.transcript);
    let submitted = dir.join("submission.json").is_file();
    let (status, error) = status(&transcript, submitted, finished.timed_out);
    let result = transcript.result.as_ref();
    let summary = Summary {
        model: transcript
            .model
            .clone()
            .unwrap_or_else(|| opts.model.clone()),
        level: level.to_owned(),
        arm: opts.arm,
        status,
        started_unix_s,
        duration_s,
        cost_usd: result.and_then(|r| r.total_cost_usd),
        turns: result.and_then(|r| r.num_turns),
        tool_calls: u32::try_from(transcript.tool_calls.len()).unwrap_or(u32::MAX),
        foreign_tools: transcript.foreign_tools(),
        result_subtype: result.map(|r| r.subtype.clone()),
        exit_code: finished.exit_code,
        error,
    };
    write_summary(&dir, &summary)?;
    Ok((dir, summary))
}

/// One line per run: level, directory, status, verdict, cost, and time.
fn run_line(dir: &Path, summary: &Summary) -> String {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let verdict = match std::fs::read_to_string(dir.join("submission.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    {
        Some(s) if s["outcome"]["goal_met"] == true => "goal_met",
        Some(_) => "missed",
        None => "-",
    };
    let status = serde_json::to_value(summary.status)
        .ok()
        .and_then(|v| v.as_str().map(ToOwned::to_owned))
        .unwrap_or_default();
    format!(
        "{} {name} {status} {verdict} {:.2} USD {:.0} s",
        summary.level,
        summary.cost_usd.unwrap_or(0.0),
        summary.duration_s
    )
}
