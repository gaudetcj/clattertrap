//! The arena runner: drives a contestant through the MCP tools and saves every run. Providers:
//! Claude Code headless, and a chat endpoint.

mod claude;
mod provider;
mod status;
mod stream;

#[cfg(test)]
mod tests;

pub use claude::ClaudeCode;
pub use provider::{Provider, Session};

use crate::catalog;
use crate::mcp::arm::Arm;
use crate::records::transcript::{Record, to_jsonl};
use crate::records::{Summary, write_summary};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// The MCP server name in the per-run config; Claude Code names its tools `mcp__clattertrap__<tool>`.
pub const SERVER_NAME: &str = "clattertrap";

/// The contestant prompt every model gets, with `{level}` and `{run_budget}` to fill.
const PROMPT: &str = include_str!("../../../../pilot/prompt.txt");

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

/// What one `arena` invocation runs.
#[derive(Debug)]
pub struct Options {
    /// The model id as the provider knows it; `model_slug` of it names the run directories.
    pub model: String,
    /// Sessions per level.
    pub runs: u32,
    /// The arm.
    pub arm: Arm,
    /// One level instead of all of them.
    pub level: Option<String>,
    /// Where run directories go.
    pub out: PathBuf,
}

/// A model id as a directory name: lowercase, every run of characters outside `[a-z0-9.]` becomes
/// one `-`, and the ends are trimmed of `-`. Claude ids come through unchanged.
#[must_use]
pub fn model_slug(id: &str) -> String {
    let mut out = String::new();
    for c in id.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() || c == '.' {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
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

/// The arguments that make this binary serve `level` under `arm`, recording into `dir`.
#[must_use]
pub fn server_args(level: &str, arm: Arm, dir: &Path) -> Vec<String> {
    [
        "mcp",
        "--level",
        level,
        "--arm",
        &arm.to_string(),
        "--out",
        &dir.display().to_string(),
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// The per-run MCP config: this binary serving `level` under `arm`, recording into `dir`.
#[must_use]
pub fn per_run_config(exe: &Path, level: &str, arm: Arm, dir: &Path) -> Value {
    json!({
        "mcpServers": {
            SERVER_NAME: {
                "command": exe.display().to_string(),
                "args": server_args(level, arm, dir),
            }
        }
    })
}

/// Runs every level (or the one named) `runs` times through `provider`, printing one line per run
/// and, at the end, the runs to do again.
///
/// # Errors
/// When a level is unknown, or a run cannot be started or saved.
pub fn run<P: Provider>(opts: &Options, provider: &P) -> Result<()> {
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
            let (dir, summary) = runtime.block_on(one_run(opts, provider, &out, &level.id))?;
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

async fn one_run<P: Provider>(
    opts: &Options,
    provider: &P,
    out: &Path,
    level: &str,
) -> Result<(PathBuf, Summary)> {
    let slug = model_slug(&opts.model);
    let dir = next_run_dir(out, &slug, level, opts.arm)?;
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let exe = std::env::current_exe().context("finding this executable")?;
    let config_path = dir.join("mcp.json");
    std::fs::write(
        &config_path,
        serde_json::to_string_pretty(&per_run_config(&exe, level, opts.arm, &dir))?,
    )
    .with_context(|| format!("writing {}", config_path.display()))?;
    let prompt = fill_prompt(level, opts.arm.run_budget());
    let session = Session {
        dir: &dir,
        prompt: &prompt,
        wall_clock: Duration::from_secs(Arm::WALL_CLOCK_S),
        max_budget_usd: Arm::MAX_BUDGET_USD,
    };
    let started_unix_s = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let clock = Instant::now();
    let finished = provider.run(&session).await?;
    let duration_s = clock.elapsed().as_secs_f64();
    std::fs::write(dir.join("transcript.jsonl"), to_jsonl(&finished.records))
        .context("writing transcript.jsonl")?;
    std::fs::write(dir.join("provider.jsonl"), &finished.raw).context("writing provider.jsonl")?;
    let submitted = dir.join("submission.json").is_file();
    let (status, error) = status::status(&finished, submitted);
    let summary = Summary {
        model: finished.model.clone().unwrap_or(slug),
        level: level.to_owned(),
        arm: opts.arm,
        status,
        started_unix_s,
        duration_s,
        cost_usd: result_field(&finished.records, |_, _, c| c),
        turns: result_field(&finished.records, |_, t, _| t),
        tool_calls: u32::try_from(tool_calls(&finished.records)).unwrap_or(u32::MAX),
        foreign_tools: status::foreign_tools(&finished.records),
        result_subtype: result_field(&finished.records, |s, _, _| Some(s.to_owned())),
        exit_code: finished.exit_code,
        error,
    };
    write_summary(&dir, &summary)?;
    Ok((dir, summary))
}

/// One field of the last result record.
fn result_field<T>(
    records: &[Record],
    pick: impl FnOnce(&str, Option<u32>, Option<f64>) -> Option<T>,
) -> Option<T> {
    match status::last_result(records) {
        Some(Record::Result {
            subtype,
            num_turns,
            total_cost_usd,
            ..
        }) => pick(subtype, *num_turns, *total_cost_usd),
        _ => None,
    }
}

fn tool_calls(records: &[Record]) -> usize {
    records
        .iter()
        .filter(|r| matches!(r, Record::ToolUse { .. }))
        .count()
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
    format!(
        "{} {name} {} {verdict} {:.2} USD {:.0} s",
        summary.level,
        summary.status.name(),
        summary.cost_usd.unwrap_or(0.0),
        summary.duration_s
    )
}
