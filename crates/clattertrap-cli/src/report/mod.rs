//! `clattertrap report`: what a tree of pilot runs says, as tables, as CSV, or as a sample to watch.

mod text;

use crate::catalog;
use crate::records::{Status, Submission, Summary, Tool, read_calls, read_json, run_dirs};
use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

/// What one `report` invocation prints.
#[derive(Debug)]
pub struct Options {
    /// A runs or gallery directory: `summary.json` three levels down.
    pub dir: PathBuf,
    /// One CSV row per run instead of the tables.
    pub csv: bool,
    /// Print this many counted submissions, chosen with `seed`, instead of the tables.
    pub sample: Option<usize>,
    /// The seed for `sample`.
    pub seed: u64,
}

/// The scene-taking calls of one run, counted.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Calls {
    /// `run` calls, accepted or not.
    pub attempts: u32,
    /// Calls whose scene passed validation.
    pub accepted: u32,
    /// Calls whose scene the rules rejected.
    pub rejected: u32,
}

/// What a submission contributes to the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submitted {
    /// Whether the submitted machine met the goal.
    pub goal_met: bool,
    /// The machine name.
    pub name: String,
}

/// One run directory, read.
#[derive(Debug)]
pub struct Run {
    /// The last three path components, `a/b/arm-NN`, with forward slashes.
    pub rel: String,
    /// Its `summary.json`.
    pub summary: Summary,
    /// Its `submission.json`, when there is one.
    pub submission: Option<Submitted>,
    /// Its `calls/`.
    pub calls: Calls,
}

impl Run {
    /// Whether the run counts: a result of the contestant's own making.
    #[must_use]
    pub const fn counts(&self) -> bool {
        self.summary.status.counts()
    }

    /// Whether a counted submission met the goal.
    #[must_use]
    pub fn solved(&self) -> bool {
        self.counts() && self.submission.as_ref().is_some_and(|s| s.goal_met)
    }
}

/// Prints the report.
///
/// # Errors
/// When the directory holds no runs or a record cannot be read.
pub fn run(opts: &Options) -> Result<()> {
    let runs = load(&opts.dir)?;
    ensure!(
        !runs.is_empty(),
        "no runs under {}: expected <a>/<b>/<c>/summary.json three levels down",
        opts.dir.display()
    );
    let tiers = catalog::levels()?
        .into_iter()
        .map(|l| (l.id, l.tier))
        .collect::<Vec<_>>();
    let out = if let Some(n) = opts.sample {
        text::sample(&runs, n, opts.seed)
    } else if opts.csv {
        text::csv(&runs)
    } else {
        text::tables(&runs, &tiers)
    };
    print!("{out}");
    Ok(())
}

/// Every run under `dir`, in path order, from either layout: the summary names the model, level,
/// and arm.
fn load(dir: &Path) -> Result<Vec<Run>> {
    run_dirs(dir)
        .iter()
        .map(|run_dir| read_run(run_dir))
        .collect()
}

/// The last three components of `dir`, joined with forward slashes.
fn relative_name(dir: &Path) -> String {
    let mut parts: Vec<String> = dir
        .components()
        .rev()
        .take(3)
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts.reverse();
    parts.join("/")
}

fn read_run(dir: &Path) -> Result<Run> {
    let rel = relative_name(dir);
    let summary: Summary = read_json(&dir.join("summary.json"))?;
    let submission = match read_json::<Submission>(&dir.join("submission.json")) {
        Ok(s) => Some(Submitted {
            goal_met: s.outcome.goal_met,
            name: s.name,
        }),
        Err(_) if !dir.join("submission.json").exists() => None,
        Err(e) => return Err(e),
    };
    Ok(Run {
        rel,
        summary,
        submission,
        calls: count_calls(dir)?,
    })
}

fn count_calls(dir: &Path) -> Result<Calls> {
    let mut counts = Calls::default();
    for (_, call) in read_calls(dir)? {
        let call = call?;
        if call.tool == Tool::Run {
            counts.attempts += 1;
        }
        if call.accepted() {
            counts.accepted += 1;
        } else {
            counts.rejected += 1;
        }
    }
    Ok(counts)
}
