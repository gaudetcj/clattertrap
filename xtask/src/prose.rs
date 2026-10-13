//! Prose lint: flags the banned words, em dashes, and emoji listed in the writing rules.
//!
//! The banned list is assembled from fragments so this file passes its own lint.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// One finding: the 1-based line and what was found there.
#[derive(Debug, PartialEq, Eq)]
pub struct Hit {
    /// 1-based line number.
    pub line: usize,
    /// The banned word or phrase, or `em dash` or `emoji`.
    pub what: String,
}

const FRAGMENTS: &[&[&str]] = &[
    &["seam", "less"],
    &["rob", "ust"],
    &["lever", "age"],
    &["power", "ful"],
    &["compre", "hensive"],
    &["cutting", "-edge"],
    &["state-of", "-the-art"],
    &["blaz", "ing"],
    &["effort", "less"],
    &["eleg", "ant"],
    &["del", "ve"],
    &["unl", "ock"],
    &["empo", "wer"],
    &["stream", "line"],
    &["game-", "changer"],
    &["it's worth", " noting"],
    &["it's important", " to note"],
    &["under the", " hood"],
];

fn banned() -> Vec<String> {
    FRAGMENTS.iter().map(|parts| parts.concat()).collect()
}

fn is_emoji(c: char) -> bool {
    matches!(u32::from(c), 0x1F000..=0x1FAFF | 0x2600..=0x27BF)
}

/// Lints one text and returns every hit, one per banned item per line.
#[must_use]
pub fn lint_text(text: &str) -> Vec<Hit> {
    let words = banned();
    let em_dash = char::from_u32(0x2014).unwrap_or('-');
    let curly = char::from_u32(0x2019).unwrap_or('\'');
    let mut hits = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let lower = raw.to_lowercase().replace(curly, "'");
        for w in &words {
            if lower.contains(w.as_str()) {
                hits.push(Hit {
                    line,
                    what: w.clone(),
                });
            }
        }
        if raw.contains(em_dash) {
            hits.push(Hit {
                line,
                what: "em dash".to_owned(),
            });
        }
        if raw.chars().any(is_emoji) {
            hits.push(Hit {
                line,
                what: "emoji".to_owned(),
            });
        }
    }
    hits
}

const EXTENSIONS: &[&str] = &[
    "md", "rs", "ts", "toml", "yml", "yaml", "html", "css", "txt", "json",
];

/// Whether the lint reads this repository path: prose and source, never `CLAUDE.md`, `gallery/`, lockfiles, or binaries.
#[must_use]
pub fn should_scan(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name == "CLAUDE.md" || name == "package-lock.json" {
        return false;
    }
    if path
        .components()
        .next()
        .is_some_and(|c| c.as_os_str() == "gallery")
    {
        return false;
    }
    if name == "justfile" {
        return true;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e))
}

/// Lints every tracked or untracked, non-ignored file under `root`, prints `path:line: what` per hit, and returns the hit count.
///
/// # Errors
/// Fails when `git ls-files` cannot run or a listed file cannot be read.
pub fn run(root: &Path) -> Result<usize> {
    let out = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    anyhow::ensure!(
        out.status.success(),
        "git ls-files failed with {}",
        out.status
    );
    let mut count = 0;
    for rel in String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
    {
        let path = Path::new(rel);
        if !should_scan(path) {
            continue;
        }
        let bytes = std::fs::read(root.join(path)).with_context(|| format!("reading {rel}"))?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        for hit in lint_text(&text) {
            println!("{rel}:{}: {}", hit.line, hit.what);
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::{Hit, lint_text, should_scan};
    use std::path::Path;

    fn join(parts: &[&str]) -> String {
        parts.concat()
    }

    fn lines(hits: &[Hit]) -> Vec<usize> {
        hits.iter().map(|h| h.line).collect()
    }

    #[test]
    fn flags_a_banned_word_with_its_line_number_ignoring_case() {
        let text = format!("first line\nA {} plan", join(&["Rob", "ust"]));
        let hits = lint_text(&text);
        assert_eq!(lines(&hits), vec![2]);
        assert_eq!(hits[0].what, join(&["rob", "ust"]));
    }

    #[test]
    fn flags_a_banned_phrase() {
        let text = format!("{} that x\n", join(&["it's worth", " noting"]));
        assert_eq!(lines(&lint_text(&text)), vec![1]);
    }

    #[test]
    fn flags_an_em_dash() {
        let text = format!("a {} b", char::from_u32(0x2014).unwrap());
        let hits = lint_text(&text);
        assert_eq!(lines(&hits), vec![1]);
        assert_eq!(hits[0].what, "em dash");
    }

    #[test]
    fn flags_an_emoji() {
        let text = format!("done {}", char::from_u32(0x1F600).unwrap());
        let hits = lint_text(&text);
        assert_eq!(lines(&hits), vec![1]);
        assert_eq!(hits[0].what, "emoji");
    }

    #[test]
    fn reports_every_hit_on_a_line() {
        let text = format!("{} and {}", join(&["del", "ve"]), join(&["unl", "ock"]));
        assert_eq!(lint_text(&text).len(), 2);
    }

    #[test]
    fn clean_text_passes() {
        assert!(lint_text("A ball rolls 40 cm in 1.2 s.\nThe bell rings.\n").is_empty());
    }

    #[test]
    fn scans_prose_and_source_but_not_claude_md_gallery_or_binaries() {
        assert!(should_scan(Path::new("README.md")));
        assert!(should_scan(Path::new("docs/physics.md")));
        assert!(should_scan(Path::new("crates/clattertrap-core/src/lib.rs")));
        assert!(should_scan(Path::new("web/src/main.ts")));
        assert!(should_scan(Path::new("justfile")));
        assert!(should_scan(Path::new(".github/workflows/ci.yml")));
        assert!(!should_scan(Path::new("CLAUDE.md")));
        assert!(!should_scan(Path::new("gallery/bell/opus/notes.md")));
        assert!(!should_scan(Path::new("docs/hero.gif")));
        assert!(!should_scan(Path::new("web/package-lock.json")));
    }
}
