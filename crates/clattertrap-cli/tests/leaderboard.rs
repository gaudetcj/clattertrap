//! `clattertrap leaderboard` over the fixture runs: levels solved per model and arm, ranked, with
//! aborted and flagged runs left out.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{clattertrap, fixtures, stdout};

fn runs() -> String {
    fixtures().join("runs").to_string_lossy().into_owned()
}

#[test]
fn the_table_ranks_models_by_blueprint_then_workshop_levels_solved() {
    let out = stdout(&["leaderboard", &runs()]);
    let lines: Vec<&str> = out.lines().collect();
    // Opus: two blueprint levels solved of two attempted, one of them hard; the flagged grand-tour
    // run is excluded and the no-submission one counts as a workshop miss.
    assert!(lines[1].starts_with("claude-opus-5-5"), "{out}");
    let opus: Vec<&str> = lines[1].split_whitespace().collect();
    assert_eq!(&opus[1..], ["2/2", "0/1", "1/1", "0/1", "2.73"], "{out}");
    // Haiku: blueprint missed bell-drop, workshop solved it; the aborted seesaw run is excluded.
    let haiku: Vec<&str> = lines[2].split_whitespace().collect();
    assert_eq!(&haiku[1..], ["0/1", "1/1", "0/0", "0/0", "0.38"], "{out}");
    insta::assert_snapshot!(out);
}

#[test]
fn the_markdown_form_is_a_table_with_a_count_line_for_the_readme() {
    let out = stdout(&["leaderboard", &runs(), "--markdown"]);
    assert!(
        out.starts_with(
            "| model | blueprint | workshop | hard blueprint | hard workshop | mean USD |\n|---|"
        ),
        "{out}"
    );
    assert!(
        out.contains("| claude-opus-5-5 | 2/2 | 0/1 | 1/1 | 0/1 | 2.73 |"),
        "{out}"
    );
    assert!(
        out.contains(
            "5 counted runs by 2 models on rules version 1, 4 of them with a machine and 1 without a submission."
        ),
        "{out}"
    );
    insta::assert_snapshot!(out);
}

#[test]
fn an_empty_or_missing_directory_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = clattertrap(&["leaderboard", dir.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("no runs"));
}
