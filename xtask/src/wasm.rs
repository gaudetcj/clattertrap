//! The wasm recipes: build `clattertrap-wasm` into `web/pkg/` and run the wasm32 test legs, each behind
//! a check that the pinned tool version is installed.

use crate::{cargo, step};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// The wasmtime release `just wasm-test` runs core's tests under.
pub const WASMTIME_VERSION: &str = "49.0.2";

/// The version of a package in `Cargo.lock`, by exact name.
#[must_use]
pub fn locked_version(lock: &str, name: &str) -> Option<String> {
    let header = format!("name = \"{name}\"");
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == header {
            return lines
                .next()?
                .trim()
                .strip_prefix("version = \"")?
                .strip_suffix('"')
                .map(str::to_owned);
        }
    }
    None
}

/// Whether a `--version` line names the expected version as one whole token.
#[must_use]
pub fn reports_version(output: &str, expected: &str) -> bool {
    output.split_whitespace().any(|t| t == expected)
}

fn require_tool(tool: &str, expected: &str, install: &str) -> Result<()> {
    let out = Command::new(tool)
        .arg("--version")
        .output()
        .with_context(|| format!("{tool} is not installed; {install}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    anyhow::ensure!(
        reports_version(&text, expected),
        "{tool} {expected} is required, found {:?}; {install}",
        text.trim()
    );
    Ok(())
}

fn wasm_bindgen_version(root: &Path) -> Result<String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).context("reading Cargo.lock")?;
    locked_version(&lock, "wasm-bindgen").context("wasm-bindgen is not in Cargo.lock")
}

fn require_wasm_bindgen(root: &Path, tool: &str) -> Result<()> {
    let version = wasm_bindgen_version(root)?;
    let install =
        format!("install it with: cargo install wasm-bindgen-cli --version {version} --locked");
    require_tool(tool, &version, &install)
}

/// Builds `clattertrap-wasm` for the browser into `web/pkg/` with the pinned `wasm-bindgen`.
///
/// # Errors
/// When the tool is missing or at another version, or a build step fails.
pub fn build(root: &Path) -> Result<()> {
    require_wasm_bindgen(root, "wasm-bindgen")?;
    let cargo = cargo();
    step(
        root,
        &cargo,
        &[
            "build",
            "-p",
            "clattertrap-wasm",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    let wasm = root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("clattertrap_wasm.wasm");
    let out = root.join("web").join("pkg");
    let (wasm, out) = (wasm.to_string_lossy(), out.to_string_lossy());
    step(
        root,
        "wasm-bindgen",
        &["--target", "web", "--out-dir", &out, &wasm],
    )
}

/// Runs core's tests on wasm32-wasip1 under the pinned wasmtime, then the wrapper's tests on
/// wasm32-unknown-unknown under the pinned wasm-bindgen-test-runner.
///
/// # Errors
/// When a tool is missing or at another version, or a test fails.
pub fn test(root: &Path) -> Result<()> {
    let install = format!(
        "install it with: cargo install wasmtime-cli --version {WASMTIME_VERSION} --locked"
    );
    require_tool("wasmtime", WASMTIME_VERSION, &install)?;
    require_wasm_bindgen(root, "wasm-bindgen-test-runner")?;
    let cargo = cargo();
    step(
        root,
        &cargo,
        &[
            "test",
            "-p",
            "clattertrap-core",
            "--target",
            "wasm32-wasip1",
        ],
    )?;
    step(
        root,
        &cargo,
        &[
            "test",
            "-p",
            "clattertrap-wasm",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = "[[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.129\"\nsource = \"registry\"\n\n[[package]]\nname = \"wasm-bindgen-macro\"\nversion = \"0.2.129\"\n";

    #[test]
    fn finds_the_locked_version_of_a_package_by_exact_name() {
        assert_eq!(
            locked_version(LOCK, "wasm-bindgen").as_deref(),
            Some("0.2.129")
        );
        assert_eq!(
            locked_version(LOCK, "wasm-bindgen-macro").as_deref(),
            Some("0.2.129")
        );
        assert_eq!(locked_version(LOCK, "wasm-bindgen-shared"), None);
    }

    #[test]
    fn a_tool_reports_its_version_as_one_whitespace_separated_token() {
        assert!(reports_version("wasm-bindgen 0.2.129", "0.2.129"));
        assert!(reports_version(
            "wasmtime 49.0.2 (abc123 2026-10-02)",
            "49.0.2"
        ));
        assert!(!reports_version("wasm-bindgen 0.2.128", "0.2.129"));
        assert!(!reports_version("wasmtime 49.0.20", "49.0.2"));
    }
}
