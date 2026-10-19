//! Project automation for Clattertrap: the `ci` driver, the prose lint, the README leaderboard, the wasm recipes, the fuzz runner,
//! and the hero recording.
#![forbid(unsafe_code)]

pub mod hero;
pub mod prose;
pub mod readme;
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
    step_env(root, program, args, &[])
}

/// Runs one tool from the repository root with extra environment variables and fails if it does.
///
/// # Errors
/// When the tool cannot start or exits with failure.
pub fn step_env(
    root: &Path,
    program: &str,
    args: &[&str],
    envs: &[(String, String)],
) -> Result<()> {
    println!("== {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
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

/// The test binary behind a `just fuzz` target.
#[must_use]
pub fn fuzz_test_name(target: &str) -> Option<&'static str> {
    match target {
        "scene" => Some("fuzz_scene"),
        _ => None,
    }
}

/// Runs a fuzz target's ignored long-form test in release mode for `secs` seconds.
///
/// # Errors
/// When the target is unknown or the test finds a panic.
pub fn fuzz(root: &Path, target: &str, secs: u64) -> Result<()> {
    let Some(test) = fuzz_test_name(target) else {
        anyhow::bail!("unknown fuzz target {target:?}; targets: scene");
    };
    println!("== fuzz {target} for {secs} s");
    let status = Command::new(cargo())
        .args([
            "test",
            "-p",
            "clattertrap-core",
            "--release",
            "--test",
            test,
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("CLATTERTRAP_FUZZ_SECS", secs.to_string())
        .current_dir(root)
        .status()
        .context("running cargo test")?;
    anyhow::ensure!(status.success(), "fuzz {target} failed with {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::fuzz_test_name;

    #[test]
    fn fuzz_targets_map_to_their_test_binaries() {
        assert_eq!(fuzz_test_name("scene"), Some("fuzz_scene"));
        assert_eq!(fuzz_test_name("levels"), None);
    }
}
