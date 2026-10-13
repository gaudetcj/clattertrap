//! The CLI surface is part of the interface models and people use, so its help text is snapshotted.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

#[test]
fn help_lists_the_cli_surface() {
    let out = Command::new(env!("CARGO_BIN_EXE_clattertrap"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(out.status.success(), "exit status {}", out.status);
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
