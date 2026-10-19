//! `cargo xtask site [BASE]`: builds the wasm package and the viewer into `web/dist` for hosting
//! under `BASE`, `/clattertrap/` by default, which is what the Pages workflow deploys.

use crate::{step, wasm};
use anyhow::Result;
use std::path::Path;

/// The base path of the GitHub Pages project site.
pub const DEFAULT_BASE: &str = "/clattertrap/";

/// The npm arguments that build the viewer under `base`; the `--` hands the flag on to vite.
#[must_use]
pub fn build_args(base: &str) -> Vec<String> {
    ["--prefix", "web", "run", "build", "--", "--base", base]
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// Builds `web/dist` for hosting under `base`.
///
/// # Errors
/// When the wasm build or the web build fails.
pub fn build(root: &Path, base: &str) -> Result<()> {
    wasm::build(root)?;
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let args = build_args(base);
    step(
        root,
        npm,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_base_path_reaches_vite_after_the_npm_separator() {
        let args = super::build_args("/clattertrap/");
        assert_eq!(
            args,
            [
                "--prefix",
                "web",
                "run",
                "build",
                "--",
                "--base",
                "/clattertrap/"
            ]
        );
    }
}
