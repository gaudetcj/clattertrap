//! `cargo xtask readme`: writes the leaderboard block of `README.md` from
//! `clattertrap leaderboard gallery --markdown`, or with `--check` fails when the block is stale.

use crate::cargo;
use anyhow::{Context, Result, ensure};
use std::path::Path;
use std::process::Command;

/// The line before the generated block.
pub const START: &str = "<!-- leaderboard:start -->";
/// The line after the generated block.
pub const END: &str = "<!-- leaderboard:end -->";

/// `readme` with everything between the markers replaced by `table`, each marker kept on its own
/// line.
///
/// # Errors
/// When a marker is missing or the end marker comes first.
pub fn splice(readme: &str, table: &str) -> Result<String> {
    let start = readme
        .find(START)
        .with_context(|| format!("README.md has no {START} line"))?;
    let body_at = start + START.len();
    let end = readme[body_at..]
        .find(END)
        .map(|i| body_at + i)
        .with_context(|| format!("README.md has no {END} line after the start marker"))?;
    Ok(format!(
        "{}{START}\n{}\n{END}{}",
        &readme[..start],
        table.trim_end(),
        &readme[end + END.len()..]
    ))
}

/// The Markdown leaderboard over `gallery/`, as the CLI prints it.
fn table(root: &Path) -> Result<String> {
    let out = Command::new(cargo())
        .args([
            "run",
            "--quiet",
            "-p",
            "clattertrap-cli",
            "--",
            "leaderboard",
            "gallery",
            "--markdown",
        ])
        .current_dir(root)
        .output()
        .context("running clattertrap leaderboard")?;
    ensure!(
        out.status.success(),
        "clattertrap leaderboard failed with {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).context("leaderboard output is not UTF-8")
}

/// Rewrites the block, or with `check` only compares and fails when it would change.
///
/// # Errors
/// When the README or the markers are missing, the CLI fails, or `check` finds the block stale.
pub fn run(root: &Path, check: bool) -> Result<()> {
    println!("== readme{}", if check { " --check" } else { "" });
    let path = root.join("README.md");
    let readme = std::fs::read_to_string(&path).context("reading README.md")?;
    let fresh = splice(&readme, &table(root)?)?;
    if fresh == readme {
        println!("README.md is current");
        return Ok(());
    }
    ensure!(!check, "README.md is stale; run: just readme");
    std::fs::write(&path, fresh).context("writing README.md")?;
    println!("README.md updated");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::{END, START, splice};

    const README: &str =
        "# Title\n\n<!-- leaderboard:start -->\nold\n<!-- leaderboard:end -->\n\nAfter.\n";

    #[test]
    fn replaces_the_block_between_the_markers_and_keeps_the_rest() {
        let out = splice(README, "| a |\n").unwrap();
        assert_eq!(out, format!("# Title\n\n{START}\n| a |\n{END}\n\nAfter.\n"));
    }

    #[test]
    fn is_idempotent() {
        let once = splice(README, "| a |\n| b |\n").unwrap();
        assert_eq!(splice(&once, "| a |\n| b |\n").unwrap(), once);
    }

    #[test]
    fn needs_both_markers_in_order() {
        assert!(splice("no markers", "x").is_err());
        assert!(splice(&format!("{END}\n{START}\n"), "x").is_err());
        assert!(splice(&format!("{START}\n"), "x").is_err());
    }
}
