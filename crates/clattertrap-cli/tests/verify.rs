//! `clattertrap verify` over a fixture gallery: one honest entry and one whose stored results went stale.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::clattertrap;
use std::path::{Path, PathBuf};

fn gallery() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gallery")
}

fn copy_entry(from: &Path, to: &Path) {
    std::fs::create_dir_all(to.join("calls")).unwrap();
    for name in ["summary.json", "submission.json"] {
        std::fs::copy(from.join(name), to.join(name)).unwrap();
    }
    for entry in std::fs::read_dir(from.join("calls")).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, to.join("calls").join(path.file_name().unwrap())).unwrap();
    }
}

#[test]
fn an_honest_entry_verifies_clean() {
    let dir = tempfile::tempdir().unwrap();
    copy_entry(
        &gallery().join("seesaw-launch/claude-opus-5-5/blueprint-01"),
        &dir.path()
            .join("seesaw-launch/claude-opus-5-5/blueprint-01"),
    );
    let out = clattertrap(&["verify", &dir.path().to_string_lossy()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "verified 1 entry, 2 runs\n"
    );
}

#[test]
fn stale_results_are_listed_per_field_and_fail_the_check() {
    let out = clattertrap(&["verify", &gallery().to_string_lossy()]);
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains(
            "bell-drop/claude-haiku-5-5/workshop-01/submission.json: steps stored 185, now 184"
        ),
        "{text}"
    );
    assert!(
        text.contains("submission.json: state_hash stored bd33e6b7c5e0f2bd, now bd33e6b7c5e0f2bc"),
        "{text}"
    );
    assert!(
        text.contains("calls/0002-run.json: time_s stored 1.5, now 1.5333334"),
        "{text}"
    );
    assert!(
        text.contains("calls/0003-submit.json: does not parse"),
        "{text}"
    );
    assert!(
        text.ends_with("verified 2 entries, 4 runs, 4 mismatches\n"),
        "{text}"
    );
    insta::assert_snapshot!(text);
}

#[test]
fn a_run_whose_status_does_not_count_is_a_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let entry = dir
        .path()
        .join("seesaw-launch/claude-opus-5-5/blueprint-01");
    copy_entry(
        &gallery().join("seesaw-launch/claude-opus-5-5/blueprint-01"),
        &entry,
    );
    let summary = std::fs::read_to_string(entry.join("summary.json")).unwrap();
    std::fs::write(
        entry.join("summary.json"),
        summary.replace("\"status\": \"submitted\"", "\"status\": \"aborted\""),
    )
    .unwrap();
    let out = clattertrap(&["verify", &dir.path().to_string_lossy()]);
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.starts_with(
            "seesaw-launch/claude-opus-5-5/blueprint-01/summary.json: status aborted does not count"
        ),
        "{text}"
    );
}

#[test]
fn a_directory_without_entries_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = clattertrap(&["verify", &dir.path().to_string_lossy()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no entries"));
}
