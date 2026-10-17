//! Writes PNGs of a scene through core's render module.

use anyhow::{Context, Result};
use clattertrap_core::level::Level;
use clattertrap_core::render::{Frame, png};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::{Problem, ValidScene, validate};
use std::path::Path;

/// Pixels per centimetre in every picture: a 400 by 300 cm level is 800 by 600.
const PX_PER_CM: f32 = 2.0;

/// Validation problems as one error, one problem per line.
pub fn problems_error(problems: &[Problem]) -> anyhow::Error {
    let lines: Vec<String> = problems.iter().map(ToString::to_string).collect();
    anyhow::anyhow!("{}", lines.join("\n"))
}

/// The simulator's current state as PNG bytes.
///
/// # Errors
/// When encoding fails.
pub fn png_bytes(level: &Level, valid: &ValidScene, sim: &Simulator) -> Result<Vec<u8>> {
    let pixmap = Frame::new(level, PX_PER_CM).draw(sim, valid);
    Ok(png(&pixmap)?)
}

/// Writes the simulator's current state to `path`.
///
/// # Errors
/// When encoding or writing the file fails.
pub fn write_png(level: &Level, valid: &ValidScene, sim: &Simulator, path: &Path) -> Result<()> {
    let bytes = png_bytes(level, valid, sim)?;
    std::fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

/// Writes a scene as it stands at the start of a run.
///
/// # Errors
/// When the scene fails validation or the file cannot be written.
pub fn write_start_png(level: &Level, scene: &Scene, path: &Path) -> Result<()> {
    let valid = validate(level, scene, &Rules::V1).map_err(|p| problems_error(&p))?;
    let sim = Simulator::new(level, &valid, &Rules::V1);
    write_png(level, &valid, &sim, path)
}
