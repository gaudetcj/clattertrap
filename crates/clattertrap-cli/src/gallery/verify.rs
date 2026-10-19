//! `clattertrap verify`: re-simulates every stored outcome under a directory of runs and reports
//! each one that no longer holds.

use crate::catalog;
use crate::records::{
    Call, Status, Submission, Summary, read_calls, read_json, run_dirs, scene_json,
};
use crate::run;
use anyhow::{Result, ensure};
use clattertrap_core::rules::Rules;
use clattertrap_core::score::Outcome;
use clattertrap_core::validate::check;
use serde_json::Value;
use std::path::Path;

/// One stored result that no longer holds, or a file that cannot be checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// The file, relative to the verified directory, with forward slashes.
    pub path: String,
    /// What differs.
    pub what: String,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.what)
    }
}

/// What a verification found.
#[derive(Debug, Default)]
pub struct Report {
    /// Run directories seen.
    pub entries: usize,
    /// Stored outcomes re-simulated.
    pub runs: usize,
    /// Everything that no longer holds, in path order.
    pub mismatches: Vec<Mismatch>,
}

impl Report {
    /// The closing count line.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let entries = plural(self.entries, "entry", "entries");
        let runs = plural(self.runs, "run", "runs");
        if self.mismatches.is_empty() {
            format!("verified {entries}, {runs}")
        } else {
            let n = plural(self.mismatches.len(), "mismatch", "mismatches");
            format!("verified {entries}, {runs}, {n}")
        }
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Verifies every `<dir>/*/*/*/summary.json` in path order.
///
/// # Errors
/// When `dir` holds no run directories.
pub fn verify(dir: &Path) -> Result<Report> {
    let mut report = Report::default();
    for entry in run_dirs(dir) {
        let rel = relative(dir, &entry);
        let (runs, mismatches) = check_entry(&entry, &rel);
        report.entries += 1;
        report.runs += runs;
        report.mismatches.extend(mismatches);
    }
    ensure!(
        report.entries > 0,
        "no entries under {}: expected <a>/<b>/<c>/summary.json three levels down",
        dir.display()
    );
    Ok(report)
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Checks one run directory: the summary, the submission, and every recorded outcome. Returns the
/// outcomes re-simulated and what did not hold.
#[must_use]
pub fn check_entry(dir: &Path, rel: &str) -> (usize, Vec<Mismatch>) {
    let at = |file: &str, what: String| Mismatch {
        path: format!("{rel}/{file}"),
        what,
    };
    let summary: Summary = match read_json(&dir.join("summary.json")) {
        Ok(s) => s,
        Err(e) => {
            return (
                0,
                vec![at(
                    "summary.json",
                    format!("does not parse: {}", e.root_cause()),
                )],
            );
        }
    };
    let mut mismatches = Vec::new();
    if !summary.status.counts() {
        mismatches.push(at(
            "summary.json",
            format!("status {} does not count", summary.status.name()),
        ));
    }
    if !summary.foreign_tools.is_empty() {
        mismatches.push(at(
            "summary.json",
            format!("called {}", summary.foreign_tools.join(", ")),
        ));
    }
    let mut runs = 0;
    let submission = dir.join("submission.json");
    if summary.status == Status::Submitted || submission.is_file() {
        match read_json::<Submission>(&submission) {
            Ok(s) if s.level != summary.level => mismatches.push(at(
                "submission.json",
                format!("level {} but the summary says {}", s.level, summary.level),
            )),
            Ok(s) => {
                runs += 1;
                mismatches.extend(
                    outcome_mismatches(&summary.level, &s.scene, &s.outcome)
                        .into_iter()
                        .map(|what| at("submission.json", what)),
                );
            }
            Err(e) => mismatches.push(at(
                "submission.json",
                format!("does not parse: {}", e.root_cause()),
            )),
        }
    }
    for (path, call) in read_calls(dir).unwrap_or_default() {
        let file = format!(
            "calls/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        match call {
            Ok(Call {
                outcome: Some(outcome),
                scene,
                ..
            }) => {
                runs += 1;
                mismatches.extend(
                    outcome_mismatches(&summary.level, &scene, &outcome)
                        .into_iter()
                        .map(|what| at(&file, what)),
                );
            }
            Ok(_) => {}
            Err(e) => mismatches.push(at(&file, format!("does not parse: {}", e.root_cause()))),
        }
    }
    (runs, mismatches)
}

/// Re-simulates `scene` on `level_id` and lists every field of `stored` that differs.
fn outcome_mismatches(level_id: &str, scene: &Value, stored: &Outcome) -> Vec<String> {
    let level = match catalog::find(level_id).and_then(catalog::level) {
        Ok(level) => level,
        Err(e) => return vec![format!("{e:#}")],
    };
    let valid = match check(&level, &scene_json(scene), &Rules::V1) {
        Ok(valid) => valid,
        Err(problems) => {
            let first = problems
                .first()
                .map(ToString::to_string)
                .unwrap_or_default();
            return vec![format!("scene no longer validates: {first}")];
        }
    };
    let (now, _, _) = run::run_to_end(&level, &valid);
    diff(stored, &now)
}

/// `<field> stored X, now Y` for every field that differs.
#[must_use]
pub fn diff(stored: &Outcome, now: &Outcome) -> Vec<String> {
    if stored == now {
        return vec![];
    }
    // Through text, so f32 fields print as the records store them, not widened to f64.
    let (Ok(a), Ok(b)) = (as_json(stored), as_json(now)) else {
        return vec!["outcome differs".to_owned()];
    };
    let (Some(a), Some(b)) = (a.as_object(), b.as_object()) else {
        return vec!["outcome differs".to_owned()];
    };
    a.iter()
        .filter(|(key, value)| b.get(*key) != Some(value))
        .map(|(key, value)| {
            let other = b.get(key).map(Value::to_string).unwrap_or_default();
            format!(
                "{key} stored {}, now {}",
                bare(&value.to_string()),
                bare(&other)
            )
        })
        .collect()
}

fn as_json(outcome: &Outcome) -> serde_json::Result<Value> {
    serde_json::from_str(&serde_json::to_string(outcome)?)
}

/// JSON strings without their quotes, so hashes read as hex.
fn bare(text: &str) -> &str {
    text.strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(text)
}
