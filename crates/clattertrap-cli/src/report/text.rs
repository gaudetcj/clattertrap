//! The report's three outputs: the two tables, the CSV, and the sample.

use super::{Run, status_name};
use crate::mcp::arm::Arm;
use std::collections::BTreeSet;
use std::fmt::Write;

/// Tiers 4 and 5 are hard; see docs/levels.md.
const HARD_TIER: u8 = 4;

/// The per-model-and-arm table, the per-level table, and the excluded runs.
#[must_use]
pub fn tables(runs: &[Run], tiers: &[(String, u8)]) -> String {
    let mut out = String::new();
    let groups: BTreeSet<(&str, Arm)> = runs
        .iter()
        .map(|r| (r.summary.model.as_str(), r.summary.arm))
        .collect();
    let total = tiers.len();
    let hard_total = tiers.iter().filter(|(_, t)| *t >= HARD_TIER).count();
    let _ = writeln!(
        out,
        "{:<17} {:<10} solved  hard  runs  aborted  flagged  mean USD  mean s",
        "model", "arm"
    );
    for (model, arm) in &groups {
        let mine: Vec<&Run> = runs
            .iter()
            .filter(|r| r.summary.model == *model && r.summary.arm == *arm)
            .collect();
        let counted: Vec<&Run> = mine.iter().copied().filter(|r| r.counts()).collect();
        let solved: BTreeSet<&str> = counted
            .iter()
            .filter(|r| r.solved())
            .map(|r| r.summary.level.as_str())
            .collect();
        let hard = solved
            .iter()
            .filter(|id| tiers.iter().any(|(l, t)| l == *id && *t >= HARD_TIER))
            .count();
        let aborted = mine
            .iter()
            .filter(|r| r.summary.status == super::Status::Aborted)
            .count();
        let flagged = mine
            .iter()
            .filter(|r| r.summary.status == super::Status::Flagged)
            .count();
        let _ = writeln!(
            out,
            "{model:<17} {:<10} {:>2}/{total:<3} {hard:>2}/{hard_total:<2} {:>4} {aborted:>8} {flagged:>8} {:>9.2} {:>7.0}",
            arm.to_string(),
            solved.len(),
            counted.len(),
            mean(counted.iter().filter_map(|r| r.summary.cost_usd)),
            mean(counted.iter().map(|r| r.summary.duration_s)),
        );
    }
    out.push('\n');
    out.push_str(&per_level(runs, tiers, &groups));
    let excluded: Vec<&Run> = runs.iter().filter(|r| !r.counts()).collect();
    if !excluded.is_empty() {
        out.push_str("\nexcluded (run again with --level):\n");
        for r in excluded {
            let _ = writeln!(
                out,
                "  {}  {}  {}",
                r.rel,
                status_name(r.summary.status),
                r.summary.error.as_deref().unwrap_or("")
            );
        }
    }
    out
}

fn per_level(runs: &[Run], tiers: &[(String, u8)], groups: &BTreeSet<(&str, Arm)>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<17} {:<10} {:<14} runs  solved  attempts  accepted  rejected  submitted",
        "model", "arm", "level"
    );
    for (model, arm) in groups {
        for (level, _) in tiers {
            let counted: Vec<&Run> = runs
                .iter()
                .filter(|r| {
                    r.summary.model == *model
                        && r.summary.arm == *arm
                        && r.summary.level == *level
                        && r.counts()
                })
                .collect();
            if counted.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "{model:<17} {:<10} {level:<14} {:>4} {:>6} {:>8} {:>8} {:>8} {:>9}",
                arm.to_string(),
                counted.len(),
                counted.iter().filter(|r| r.solved()).count(),
                counted.iter().map(|r| r.calls.attempts).sum::<u32>(),
                counted.iter().map(|r| r.calls.accepted).sum::<u32>(),
                counted.iter().map(|r| r.calls.rejected).sum::<u32>(),
                counted.iter().filter(|r| r.submission.is_some()).count(),
            );
        }
    }
    out
}

#[allow(clippy::cast_precision_loss)] // run counts are small
fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, n) = values.fold((0.0, 0usize), |(s, n), v| (s + v, n + 1));
    if n == 0 { 0.0 } else { sum / n as f64 }
}

/// One row per run, every run included, with its status.
#[must_use]
pub fn csv(runs: &[Run]) -> String {
    let mut out = String::from(
        "model,arm,level,run,status,goal_met,name,cost_usd,duration_s,turns,tool_calls,attempts,accepted,rejected,error\n",
    );
    for r in runs {
        let s = &r.summary;
        let run = r.rel.rsplit('/').next().unwrap_or("");
        let _ = writeln!(
            out,
            "{},{},{},{run},{},{},{},{},{:.1},{},{},{},{},{},{}",
            s.model,
            s.arm,
            s.level,
            status_name(s.status),
            r.submission
                .as_ref()
                .map_or(String::new(), |sub| sub.goal_met.to_string()),
            quote(r.submission.as_ref().map_or("", |sub| sub.name.as_str())),
            s.cost_usd.map_or(String::new(), |c| format!("{c:.4}")),
            s.duration_s,
            s.turns.map_or(String::new(), |t| t.to_string()),
            s.tool_calls,
            r.calls.attempts,
            r.calls.accepted,
            r.calls.rejected,
            quote(s.error.as_deref().unwrap_or("")),
        );
    }
    out
}

fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

/// `n` counted runs with a submission, chosen by a seeded generator, one path per line.
#[must_use]
pub fn sample(runs: &[Run], n: usize, seed: u64) -> String {
    let mut pool: Vec<&Run> = runs
        .iter()
        .filter(|r| r.counts() && r.submission.is_some())
        .collect();
    let mut state = seed;
    let mut out = String::new();
    for _ in 0..n.min(pool.len()) {
        // Knuth's MMIX linear congruential generator; the high bits pick the index.
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let index = usize::try_from(state >> 33).unwrap_or(0) % pool.len();
        let picked = pool.swap_remove(index);
        let _ = writeln!(out, "{}", picked.rel);
    }
    out
}
