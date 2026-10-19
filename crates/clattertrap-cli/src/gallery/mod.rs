//! The gallery: model-made machines promoted from pilot runs, and the checks that keep their stored
//! results honest. `promote` copies a counted run into `gallery/<level>/<model>/<arm>-NN` once its
//! stored outcomes still hold; `verify` re-checks every entry.

pub mod leaderboard;
pub mod transcript;
pub mod verify;

#[cfg(test)]
mod tests;

use crate::records::{Status, Summary, read_json};
use anyhow::{Context, Result, bail, ensure};
use std::path::{Path, PathBuf};

/// What one `promote` invocation does.
#[derive(Debug)]
pub struct Options {
    /// The pilot run directories to promote.
    pub runs: Vec<PathBuf>,
    /// The gallery root.
    pub gallery: PathBuf,
}

/// Promotes every run, stopping at the first one that cannot enter.
///
/// # Errors
/// When a run does not count, cannot be replayed to its stored outcomes, lacks a file, is already in
/// the gallery, or cannot be copied.
pub fn run(opts: &Options) -> Result<()> {
    for run_dir in &opts.runs {
        let rel = promote(run_dir, &opts.gallery)
            .with_context(|| format!("promoting {}", run_dir.display()))?;
        println!("promoted {rel}");
    }
    Ok(())
}

/// Promotes one run and returns its entry path relative to the gallery.
fn promote(run_dir: &Path, gallery: &Path) -> Result<String> {
    let summary: Summary = read_json(&run_dir.join("summary.json"))?;
    let entry = entry_dir(gallery, &summary, run_dir)?;
    admit(run_dir, &summary, &entry)?;
    copy_entry(run_dir, &entry).inspect_err(|_| {
        let _ = std::fs::remove_dir_all(&entry);
    })?;
    Ok(relative(gallery, &entry))
}

/// `gallery/<level>/<model>/<run dir name>`; the run directory must carry the summary's arm.
///
/// # Errors
/// When the run directory's name does not start with the arm's prefix.
pub fn entry_dir(gallery: &Path, summary: &Summary, run_dir: &Path) -> Result<PathBuf> {
    let name = run_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    ensure!(
        name.starts_with(&format!("{}-", summary.arm.dir_prefix())),
        "run directory {name} does not start with {}",
        summary.arm.dir_prefix()
    );
    Ok(gallery.join(&summary.level).join(&summary.model).join(name))
}

/// Refuses a run the gallery must not hold.
fn admit(run_dir: &Path, summary: &Summary, entry: &Path) -> Result<()> {
    ensure!(
        summary.status.counts(),
        "status {} does not count; run it again",
        summary.status.name()
    );
    ensure!(
        summary.foreign_tools.is_empty(),
        "called {}; a flagged run never enters the gallery",
        summary.foreign_tools.join(", ")
    );
    let has_submission = run_dir.join("submission.json").is_file();
    if summary.status == Status::Submitted {
        ensure!(has_submission, "submission.json is missing");
    } else {
        ensure!(
            !has_submission,
            "submission.json is present but the status is not submitted"
        );
    }
    ensure!(
        run_dir.join("transcript.jsonl").is_file(),
        "transcript.jsonl is missing"
    );
    ensure!(
        !entry.exists(),
        "{} is already in the gallery; remove it with git first",
        entry.display()
    );
    let (_, mismatches) = verify::check_entry(run_dir, &relative(run_dir, run_dir));
    if let Some(first) = mismatches.first() {
        bail!(
            "stored outcome no longer holds: {}; run it again, never edit it",
            first.to_string().trim_start_matches('/')
        );
    }
    Ok(())
}

/// Copies the records and writes the transcript export. The runner's `mcp.json` and `stderr.txt`
/// stay behind.
fn copy_entry(run_dir: &Path, entry: &Path) -> Result<()> {
    let calls = entry.join("calls");
    std::fs::create_dir_all(&calls).with_context(|| format!("creating {}", calls.display()))?;
    for name in ["summary.json", "submission.json"] {
        let from = run_dir.join(name);
        if from.is_file() {
            std::fs::copy(&from, entry.join(name))
                .with_context(|| format!("copying {}", from.display()))?;
        }
    }
    for (path, _) in crate::records::read_calls(run_dir)? {
        let name = path.file_name().unwrap_or_default();
        std::fs::copy(&path, calls.join(name))
            .with_context(|| format!("copying {}", path.display()))?;
    }
    let stream = std::fs::read_to_string(run_dir.join("transcript.jsonl"))
        .context("reading transcript.jsonl")?;
    let export = transcript::to_jsonl(&transcript::export(&stream));
    std::fs::write(entry.join("transcript.jsonl"), export).context("writing transcript.jsonl")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
