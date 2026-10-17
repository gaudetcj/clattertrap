//! `Level` and `Scene` to `ValidScene`, or a list of `Problem`s. The only place raw input is accepted.

mod checks;
mod ranges;

use crate::events::PartId;
use crate::level::Level;
use crate::rules::Rules;
use crate::scene::{Placed, Scene};
use crate::world::{Overlap, World};
use serde::de::DeserializeOwned;
use std::fmt;

/// Name reserved for the floor body.
pub const FLOOR_NAME: &str = "floor";

/// One thing wrong with a scene: the JSON path, and a message naming the value, the rule, and a fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// JSON path such as `parts[3] (beam b2)`.
    pub path: String,
    /// What is wrong and one concrete fix.
    pub message: String,
}

impl Problem {
    fn new(path: impl Into<String>, message: String) -> Self {
        Self {
            path: path.into(),
            message,
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// A scene that passed validation against its level. Fixed objects come first, then the scene's parts.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidScene {
    names: Vec<String>,
    placed: Vec<Placed>,
    fixed_count: usize,
}

impl ValidScene {
    /// Part names by `PartId`, with the floor last.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Every placed object, level fixtures first.
    #[must_use]
    pub fn placed(&self) -> &[Placed] {
        &self.placed
    }

    /// The id of a named part or the floor.
    #[must_use]
    pub fn part_id(&self, name: &str) -> Option<PartId> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(PartId)
    }

    /// The floor's id.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn floor_id(&self) -> PartId {
        PartId((self.names.len() - 1) as u16)
    }

    /// How many parts the contestant placed.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn scene_part_count(&self) -> u32 {
        (self.placed.len() - self.fixed_count) as u32
    }

    /// JSON path of a placed part: `fixed[i] (kind id)` or `parts[i] (kind id)`.
    #[must_use]
    pub fn path(&self, part: PartId) -> String {
        let i = usize::from(part.0);
        match self.placed.get(i) {
            Some(p) if i < self.fixed_count => format!("fixed[{i}] ({} {})", p.spec.kind(), p.id),
            Some(p) => format!(
                "parts[{}] ({} {})",
                i - self.fixed_count,
                p.spec.kind(),
                p.id
            ),
            None => FLOOR_NAME.to_owned(),
        }
    }
}

/// Parses JSON into `T`, naming the JSON path of the first error, or `root` for errors at the top.
fn parse_json<T: DeserializeOwned>(json: &str, root: &str) -> Result<T, Problem> {
    let mut de = serde_json::Deserializer::from_str(json);
    serde_path_to_error::deserialize(&mut de).map_err(|e| {
        let path = e.path().to_string();
        Problem::new(
            // An empty path prints as "." and an error before any field as "?": both are the root.
            if path == "." || path == "?" {
                root.to_owned()
            } else {
                path
            },
            format!("{}; write what the format asks for there", e.inner()),
        )
    })
}

/// Parses level JSON.
///
/// # Errors
/// One problem naming the JSON path of the first thing the parser rejected.
pub fn parse_level(json: &str) -> Result<Level, Problem> {
    parse_json(json, "level")
}

/// Parses scene JSON.
///
/// # Errors
/// One problem naming the JSON path of the first thing the parser rejected.
pub fn parse_scene(json: &str) -> Result<Scene, Problem> {
    parse_json(json, "scene")
}

/// Parses scene JSON and checks it against its level and the rules, as `validate` does.
///
/// # Errors
/// The parse problem alone when the JSON does not parse, otherwise every validation problem.
pub fn check(level: &Level, scene_json: &str, rules: &Rules) -> Result<ValidScene, Vec<Problem>> {
    let scene = parse_scene(scene_json).map_err(|p| vec![p])?;
    validate(level, &scene, rules)
}

/// Checks a scene against its level and the rules.
///
/// # Errors
/// Returns every problem found, so a contestant can fix them all at once.
pub fn validate(level: &Level, scene: &Scene, rules: &Rules) -> Result<ValidScene, Vec<Problem>> {
    let mut problems = Vec::new();
    checks::level_header(level, scene, &mut problems);
    let names = checks::names(level, scene, &mut problems);
    checks::inventory(level, scene, &mut problems);
    checks::zones(level, scene, &mut problems);
    ranges::ranges(level, scene, rules, &mut problems);
    checks::links(level, scene, &names, &mut problems);
    checks::goal_target(level, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let mut names = names;
    names.push(FLOOR_NAME.to_owned());
    let placed = level.fixed.iter().chain(&scene.parts).cloned().collect();
    let valid = ValidScene {
        names,
        placed,
        fixed_count: level.fixed.len(),
    };
    let overlaps = World::build(level, &valid, rules).initial_overlaps();
    problems.extend(overlaps.into_iter().map(|o| overlap_problem(&valid, o)));
    if problems.is_empty() {
        Ok(valid)
    } else {
        Err(problems)
    }
}

/// Blames the later-placed part, or the non-floor part, and suggests lifting it clear.
fn overlap_problem(valid: &ValidScene, o: Overlap) -> Problem {
    let floor = valid.floor_id();
    let (blamed, other) = if o.b == floor || (o.a != floor && o.a > o.b) {
        (o.a, o.b)
    } else {
        (o.b, o.a)
    };
    let depth_cm = o.depth_m * 100.0;
    let y_cm = valid
        .placed
        .get(usize::from(blamed.0))
        .and_then(|p| p.spec.centre_cm())
        .map_or(0.0, |c| c[1]);
    let other_name = valid
        .names
        .get(usize::from(other.0))
        .map_or(FLOOR_NAME, String::as_str);
    Problem::new(
        valid.path(blamed),
        format!(
            "overlaps {other_name} by {depth_cm:.1} cm at the start; move it up to y_cm = {:.1}",
            y_cm + depth_cm
        ),
    )
}
