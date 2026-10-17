//! `clattertrap report` over a fixture runs tree: the tables, the CSV, and the sample.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{clattertrap, stdout};
use std::path::PathBuf;

fn runs() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("runs")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn the_report_counts_solved_levels_per_model_and_arm_and_excludes_aborted_and_flagged_runs() {
    let out = stdout(&["report", &runs()]);
    // Haiku: blueprint missed bell-drop, workshop solved it; the aborted seesaw run is excluded.
    assert!(out.contains("claude-haiku-5-5  blueprint   0/12"), "{out}");
    assert!(out.contains("claude-haiku-5-5  workshop    1/12"), "{out}");
    // Opus: two blueprint submissions solved two levels, one of them hard.
    assert!(
        out.contains("claude-opus-5-5   blueprint   2/12   1/3"),
        "{out}"
    );
    // Opus workshop: the flagged run is excluded, the no-submission run counts as a miss.
    assert!(out.contains("claude-opus-5-5   workshop    0/12"), "{out}");
    assert!(out.contains("excluded"), "{out}");
    assert!(out.contains("called Bash"), "{out}");
    assert!(out.contains("529 overloaded"), "{out}");
    insta::assert_snapshot!(out);
}

#[test]
fn the_per_level_table_separates_attempts_from_accepted_and_rejected_calls() {
    let out = stdout(&["report", &runs()]);
    // Haiku workshop on bell-drop: three run calls, one rejected; five scene calls in all.
    let line = out
        .lines()
        .find(|l| l.starts_with("claude-haiku-5-5  workshop   bell-drop"))
        .unwrap();
    let cols: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(&cols[3..], ["1", "1", "3", "4", "1", "1"], "{line}");
}

#[test]
fn the_csv_has_one_row_per_run() {
    let out = stdout(&["report", &runs(), "--csv"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 8, "{out}");
    assert!(
        lines[0].starts_with("model,arm,level,run,status,goal_met,"),
        "{out}"
    );
    insta::assert_snapshot!(out);
}

#[test]
fn the_sample_picks_submitted_runs_the_same_way_for_a_seed() {
    let a = stdout(&["report", &runs(), "--sample", "3", "--seed", "7"]);
    let b = stdout(&["report", &runs(), "--sample", "3", "--seed", "7"]);
    assert_eq!(a, b);
    let picked: Vec<&str> = a.lines().collect();
    assert_eq!(picked.len(), 3, "{a}");
    for p in &picked {
        assert!(p.contains("blueprint-0") || p.contains("workshop-0"), "{p}");
        assert!(
            !p.contains("grand-tour/workshop-01"),
            "flagged run sampled: {p}"
        );
        assert!(
            !p.contains("seesaw-launch/workshop-01"),
            "aborted run sampled: {p}"
        );
    }
    let other = stdout(&["report", &runs(), "--sample", "3", "--seed", "8"]);
    assert_ne!(a, other);
    let all = stdout(&["report", &runs(), "--sample", "10", "--seed", "1"]);
    assert_eq!(
        all.lines().count(),
        4,
        "only four runs have a counted submission: {all}"
    );
}

#[test]
fn an_empty_or_missing_directory_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = clattertrap(&["report", dir.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("no runs"));
}
