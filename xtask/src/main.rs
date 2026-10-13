//! Entry point for `cargo xtask <command>`: the ci driver, the prose lint, the wasm recipes, and stubs for later milestones.
#![forbid(unsafe_code)]

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use xtask::{cargo, step, wasm};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = repo_root();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["ci"] => ci(&root),
        ["prose"] => prose(&root),
        ["wasm"] => wasm::build(&root),
        ["wasm-test"] => wasm::test(&root),
        ["stub", name] => stub(name),
        _ => bail!("usage: cargo xtask <ci | prose | wasm | wasm-test | stub NAME>"),
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Runs the merge gate: fmt, clippy on native and wasm32, prose, nextest, doc tests, wasm tests, the wasm
/// build, web lint and tests, gallery verify.
fn ci(root: &Path) -> Result<()> {
    let cargo = cargo();
    step(root, &cargo, &["fmt", "--all", "--check"])?;
    step(
        root,
        &cargo,
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    step(
        root,
        &cargo,
        &[
            "clippy",
            "-p",
            "clattertrap-core",
            "-p",
            "clattertrap-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    prose(root)?;
    step(root, &cargo, &["nextest", "run", "--workspace"])?;
    step(root, &cargo, &["test", "--doc", "--workspace"])?;
    wasm::test(root)?;
    // The web lint type-checks against the generated web/pkg types.
    wasm::build(root)?;
    anyhow::ensure!(
        root.join("web").join("node_modules").is_dir(),
        "web/node_modules is missing; run: npm --prefix web ci"
    );
    step(root, npm(), &["--prefix", "web", "run", "lint"])?;
    step(root, npm(), &["--prefix", "web", "test"])?;
    stub("verify")?;
    println!("ci: all green");
    Ok(())
}

fn prose(root: &Path) -> Result<()> {
    println!("== prose lint");
    let hits = xtask::prose::run(root)?;
    anyhow::ensure!(hits == 0, "prose lint: {hits} hit(s)");
    Ok(())
}

/// Later milestones replace these with real recipes.
fn stub(name: &str) -> Result<()> {
    let milestone = match name {
        "fuzz" => 4,
        "pilot" => 5,
        "verify" | "hero" => 6,
        _ => bail!("unknown stub {name}"),
    };
    println!("{name}: stub until milestone {milestone}");
    Ok(())
}

fn npm() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}
