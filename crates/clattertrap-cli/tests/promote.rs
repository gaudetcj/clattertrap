//! `clattertrap promote` over a raw pilot run copied into a tempdir: what enters the gallery and what
//! is refused.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{clattertrap, copy_dir, fixtures};
use std::path::{Path, PathBuf};

/// An edit to a copied run before promoting it.
type Change = dyn Fn(&Path);

const RUN: &str = "claude-opus-5-5/seesaw-launch/blueprint-01";
const ENTRY: &str = "seesaw-launch/claude-opus-5-5/blueprint-01";

/// A tempdir holding `pilot/<RUN>` and the path of an empty `gallery/` beside it.
fn scratch() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("pilot").join(RUN);
    copy_dir(&fixtures().join("pilot").join(RUN), &run);
    let gallery = dir.path().join("gallery");
    (dir, run, gallery)
}

fn promote(run: &Path, gallery: &Path) -> std::process::Output {
    clattertrap(&[
        "promote",
        &run.to_string_lossy(),
        "--gallery",
        &gallery.to_string_lossy(),
    ])
}

fn edit(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{from} not in {}", path.display());
    std::fs::write(path, text.replace(from, to)).unwrap();
}

fn files_under(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            out.extend(
                files_under(&path)
                    .into_iter()
                    .map(|f| format!("{name}/{f}")),
            );
        } else {
            out.push(name);
        }
    }
    out.sort();
    out
}

#[test]
fn a_promoted_run_matches_the_gallery_fixture_byte_for_byte() {
    let (_dir, run, gallery) = scratch();
    let out = promote(&run, &gallery);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("promoted {ENTRY}\n")
    );
    let entry = gallery.join(ENTRY);
    let expected = fixtures().join("gallery").join(ENTRY);
    assert_eq!(
        files_under(&entry),
        [
            "calls/0001-check.json",
            "calls/0002-look.json",
            "calls/0003-check.json",
            "calls/0004-look.json",
            "calls/0005-submit.json",
            "submission.json",
            "summary.json",
            "transcript.jsonl",
        ]
    );
    assert_eq!(files_under(&entry), files_under(&expected));
    for file in files_under(&entry) {
        assert_eq!(
            std::fs::read(entry.join(&file)).unwrap(),
            std::fs::read(expected.join(&file)).unwrap(),
            "{file}"
        );
    }
}

#[test]
fn the_exported_transcript_holds_no_pictures_or_local_paths() {
    let (_dir, run, gallery) = scratch();
    assert!(promote(&run, &gallery).status.success());
    let text = std::fs::read_to_string(gallery.join(ENTRY).join("transcript.jsonl")).unwrap();
    for banned in [
        "base64",
        "iVBOR",
        "cwd",
        "Projects",
        "C:",
        "slash_commands",
        "thinking",
    ] {
        assert!(!text.contains(banned), "{banned} in {text}");
    }
    assert!(
        text.contains(r#"{"type":"image","omitted":true}"#),
        "{text}"
    );
    assert!(
        text.contains(r#""name":"mcp__clattertrap__submit""#),
        "{text}"
    );
}

#[test]
fn an_entry_is_never_overwritten() {
    let (_dir, run, gallery) = scratch();
    assert!(promote(&run, &gallery).status.success());
    let summary = gallery.join(ENTRY).join("summary.json");
    let before = std::fs::read(&summary).unwrap();
    edit(&run.join("summary.json"), "\"turns\": 8", "\"turns\": 9");
    let out = promote(&run, &gallery);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains("already in the gallery; remove it with git first")
    );
    assert_eq!(std::fs::read(&summary).unwrap(), before);
}

#[test]
fn runs_that_do_not_count_or_cannot_be_replayed_are_refused() {
    let cases: [(&str, &Change, &str); 5] = [
        (
            "aborted",
            &|run| {
                edit(
                    &run.join("summary.json"),
                    "\"status\": \"submitted\"",
                    "\"status\": \"aborted\"",
                );
            },
            "status aborted does not count",
        ),
        (
            "flagged",
            &|run| {
                edit(
                    &run.join("summary.json"),
                    "\"status\": \"submitted\"",
                    "\"status\": \"flagged\"",
                );
                edit(
                    &run.join("summary.json"),
                    "\"foreign_tools\": []",
                    "\"foreign_tools\": [\"Bash\"]",
                );
            },
            "status flagged does not count",
        ),
        (
            "missing submission",
            &|run| std::fs::remove_file(run.join("submission.json")).unwrap(),
            "submission.json is missing",
        ),
        (
            "stale outcome",
            &|run| {
                edit(
                    &run.join("submission.json"),
                    "\"steps\": 78",
                    "\"steps\": 79",
                );
            },
            "submission.json: steps stored 79, now 78",
        ),
        (
            "missing transcript",
            &|run| std::fs::remove_file(run.join("transcript.jsonl")).unwrap(),
            "transcript.jsonl is missing",
        ),
    ];
    for (name, change, message) in cases {
        let (_dir, run, gallery) = scratch();
        change(&run);
        let out = promote(&run, &gallery);
        assert_eq!(out.status.code(), Some(2), "{name}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(message), "{name}: {stderr}");
        assert!(!gallery.join(ENTRY).exists(), "{name} left an entry");
    }
}

#[test]
fn the_run_directory_must_carry_its_arm() {
    let (dir, run, gallery) = scratch();
    let renamed = dir
        .path()
        .join("pilot/claude-opus-5-5/seesaw-launch/workshop-01");
    std::fs::rename(&run, &renamed).unwrap();
    let out = promote(&renamed, &gallery);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("workshop-01 does not start with blueprint"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_counted_run_without_a_submission_is_promoted() {
    let (_dir, run, gallery) = scratch();
    std::fs::remove_file(run.join("submission.json")).unwrap();
    edit(
        &run.join("summary.json"),
        "\"status\": \"submitted\"",
        "\"status\": \"no_submission\"",
    );
    let out = promote(&run, &gallery);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(gallery.join(ENTRY).join("summary.json").is_file());
    assert!(!gallery.join(ENTRY).join("submission.json").exists());
}
