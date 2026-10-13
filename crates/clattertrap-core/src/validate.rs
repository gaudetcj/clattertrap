//! `Level` and `Scene` to `ValidScene`, or a list of `Problem`s. The only place raw input is accepted.

use crate::events::PartId;
use crate::level::Level;
use crate::parts;
use crate::rules::Rules;
use crate::scene::{Placed, Scene};
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
}

fn path_of(prefix: &str, index: usize, p: &Placed) -> String {
    format!("{prefix}[{index}] ({} {})", p.spec.kind(), p.id)
}

/// Checks a scene against its level and the rules.
///
/// # Errors
/// Returns every problem found, so a contestant can fix them all at once.
pub fn validate(level: &Level, scene: &Scene, rules: &Rules) -> Result<ValidScene, Vec<Problem>> {
    let _ = rules;
    let mut problems = Vec::new();
    if scene.level != level.id {
        problems.push(Problem {
            path: "level".to_owned(),
            message: format!(
                "is {:?} but this level is {:?}; set level to {:?}",
                scene.level, level.id, level.id
            ),
        });
    }
    let mut names: Vec<String> = Vec::with_capacity(level.fixed.len() + scene.parts.len() + 1);
    for (prefix, list) in [("fixed", &level.fixed), ("parts", &scene.parts)] {
        for (i, p) in list.iter().enumerate() {
            let path = path_of(prefix, i, p);
            if p.id == FLOOR_NAME {
                problems.push(Problem {
                    path: path.clone(),
                    message: format!("id {FLOOR_NAME:?} is reserved; rename the part"),
                });
            }
            if names.contains(&p.id) {
                problems.push(Problem {
                    path: path.clone(),
                    message: format!("id {:?} is already used; give this part a new id", p.id),
                });
            }
            if prefix == "parts" && p.spec.is_goal_object() {
                problems.push(Problem {
                    path: path.clone(),
                    message: format!(
                        "{} is a goal object only the level may place; remove it",
                        p.spec.kind()
                    ),
                });
            }
            if !parts::SUPPORTED.contains(&p.spec.kind()) {
                problems.push(Problem {
                    path,
                    message: format!("{} is not simulated yet; use another kind", p.spec.kind()),
                });
            }
            names.push(p.id.clone());
        }
    }
    if u16::try_from(names.len() + 1).is_err() {
        problems.push(Problem {
            path: "parts".to_owned(),
            message: "too many parts; use fewer than 65535".to_owned(),
        });
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    names.push(FLOOR_NAME.to_owned());
    let placed = level.fixed.iter().chain(&scene.parts).cloned().collect();
    Ok(ValidScene {
        names,
        placed,
        fixed_count: level.fixed.len(),
    })
}
