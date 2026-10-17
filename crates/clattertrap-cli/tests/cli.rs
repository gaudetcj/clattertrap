//! The read-only commands: `rules`, `levels`, and `level`. Their text is what contestants read, so it is
//! snapshotted.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{clattertrap, level_dirs, stdout};

#[test]
fn rules_prints_the_contestant_rules_in_centimetres() {
    let out = stdout(&["rules"]);
    assert!(out.starts_with("Clattertrap rules v1"), "{out}");
    assert!(out.contains("length_cm (20..150)"), "{out}");
    assert!(out.contains("steel"), "{out}");
    assert!(!out.contains("_m "), "metres leaked: {out}");
    insta::assert_snapshot!(out);
}

#[test]
fn rules_schema_prints_the_scene_schema() {
    let out = stdout(&["rules", "--schema"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["title"], "Scene");
    insta::assert_snapshot!(out);
}

#[test]
fn rules_level_schema_prints_the_level_schema() {
    let out = stdout(&["rules", "--level-schema"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["title"], "Level");
    insta::assert_snapshot!(out);
}

#[test]
fn levels_lists_every_level_directory_by_tier() {
    let out = stdout(&["levels"]);
    let listed: Vec<&str> = out
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let mut dirs = level_dirs();
    dirs.sort();
    let mut sorted = listed.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, dirs, "levels/ and the catalogue differ");
    let tiers: Vec<u8> = out
        .lines()
        .map(|l| l.split_whitespace().nth(1).unwrap().parse::<u8>().unwrap())
        .collect();
    assert!(tiers.windows(2).all(|w| w[0] <= w[1]), "{out}");
    insta::assert_snapshot!(out);
}

#[test]
fn level_prints_the_text_and_the_json() {
    let out = stdout(&["level", "bell-drop"]);
    assert!(
        out.starts_with("bell-drop: Ring the bell (tier 1)"),
        "{out}"
    );
    assert!(out.contains("A steel ball hangs in the air"), "{out}");
    assert!(
        out.contains("Inventory: beam 4, domino 6, seesaw 1"),
        "{out}"
    );
    assert!(out.contains("\"inventory\""), "{out}");
    insta::assert_snapshot!(out);
}

#[test]
fn level_png_writes_the_start_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("start.png");
    let out = stdout(&["level", "bell-drop", "--png", path.to_str().unwrap()]);
    assert!(out.contains("start.png"), "{out}");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"\x89PNG");
}

#[test]
fn an_unknown_level_fails_naming_the_id() {
    let out = clattertrap(&["level", "nope"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("nope"), "{err}");
    assert!(err.contains("clattertrap levels"), "{err}");
}
