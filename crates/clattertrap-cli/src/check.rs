//! Loading a scene file against its level: the shared front half of `check`, `look`, and `run`.

use crate::catalog;
use anyhow::{Context, Result};
use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::validate::{Problem, ValidScene, parse_scene, validate};
use std::path::Path;

/// A scene file read, parsed, matched to its level, and validated.
pub struct Loaded {
    /// The level the scene names.
    pub level: Level,
    /// The validated placement.
    pub valid: ValidScene,
}

/// Loads `path`: `Ok(Err(problems))` is a scene the rules reject, `Err` a file or level lookup failure.
///
/// # Errors
/// When the file cannot be read or the scene names a level the catalogue lacks.
pub fn load(path: &Path) -> Result<Result<Loaded, Vec<Problem>>> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let scene: Scene = match parse_scene(&text) {
        Ok(scene) => scene,
        Err(problem) => return Ok(Err(vec![problem])),
    };
    let level = catalog::level(catalog::find(&scene.level)?)?;
    Ok(validate(&level, &scene, &Rules::V1).map(|valid| Loaded { level, valid }))
}

/// Loads `path` and prints the problems, one per line, when the rules reject it.
///
/// # Errors
/// As `load`.
pub fn load_or_report(path: &Path) -> Result<Option<Loaded>> {
    match load(path)? {
        Ok(loaded) => Ok(Some(loaded)),
        Err(problems) => {
            for p in &problems {
                println!("{p}");
            }
            Ok(None)
        }
    }
}
