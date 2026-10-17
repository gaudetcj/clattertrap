//! Runs the built binary and finds the levels directory.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::{Command, Output};

pub mod mcp;

/// Runs `clattertrap` with `args` from the repository root.
pub fn clattertrap(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_clattertrap"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .unwrap()
}

/// Stdout of a run that must succeed.
pub fn stdout(args: &[&str]) -> String {
    let out = clattertrap(args);
    assert!(
        out.status.success(),
        "clattertrap {args:?} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The repository root, two directories above this crate.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The `levels/` directory.
pub fn levels_dir() -> PathBuf {
    repo_root().join("levels")
}

/// Every level directory name.
pub fn level_dirs() -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(levels_dir())
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().join("level.json").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    ids.sort();
    ids
}
