//! `clattertrap report`: what a tree of pilot runs says, as tables, as CSV, or as a sample to watch.

mod text;

use crate::catalog;
use crate::records::{Status, Submission, Summary, Tool, read_calls, read_json};
use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

/// What one `report` invocation prints.
#[derive(Debug)]
pub struct Options {
    /// The runs directory, `<model>/<level>/<arm>-NN` underneath.
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
    /// `model/level/arm-NN`, with forward slashes.
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
        "no runs under {}: expected <model>/<level>/<arm>-NN/summary.json",
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

/// Every run under `dir`, in path order.
fn load(dir: &Path) -> Result<Vec<Run>> {
    let mut runs = Vec::new();
    for model in subdirs(dir) {
        for level in subdirs(&model) {
            for run_dir in subdirs(&level) {
                let summary = run_dir.join("summary.json");
                if !summary.is_file() {
                    continue;
                }
                runs.push(read_run(&run_dir, &[&model, &level, &run_dir])?);
            }
        }
    }
    Ok(runs)
}

/// The directories directly under `dir`, sorted; none when `dir` is missing.
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

fn read_run(dir: &Path, parts: &[&Path]) -> Result<Run> {
    let rel = parts
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
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
