//! Project automation for Clattertrap: the `ci` driver, the prose lint, the wasm recipes, and stubs for later recipes.
#![forbid(unsafe_code)]

pub mod prose;
pub mod wasm;

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// The cargo that invoked xtask, so the pinned toolchain is reused.
#[must_use]
pub fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// Runs one tool from the repository root and fails if it does.
///
/// # Errors
/// When the tool cannot start or exits with failure.
pub fn step(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    println!("== {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| format!("running {program}"))?;
    anyhow::ensure!(
        status.success(),
        "{program} {} failed with {status}",
        args.join(" ")
    );
    Ok(())
}
