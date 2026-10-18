//! One contestant session: the assigned level, the arm's run budget, the one submission, and the
//! text and pictures each tool answers with. Nothing here knows about the protocol.

use super::arm::Arm;
use crate::records::{Call, Submission, Tool, scene_json, write_call, write_submission};
use crate::{catalog, level_text, picture, rules_text, run};
use anyhow::Result;
use clattertrap_core::level::Level;
use clattertrap_core::rules::Rules;
use clattertrap_core::scene;
use clattertrap_core::score::Outcome;
use clattertrap_core::sim::Simulator;
use clattertrap_core::validate::{Problem, ValidScene, check};
use serde_json::Value;
use std::fmt::Write;
use std::path::PathBuf;

/// A machine name may be this long.
pub const NAME_MAX_CHARS: usize = 60;
/// A prediction may be this long.
pub const PREDICTION_MAX_CHARS: usize = 280;

/// What a tool answers with: text, and for `level`, `look`, and `run` a PNG.
#[derive(Debug)]
pub struct Reply {
    /// The text the contestant reads.
    pub text: String,
    /// A picture, when the tool draws one.
    pub png: Option<Vec<u8>>,
}

impl Reply {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            png: None,
        }
    }
}

/// Why a tool did not answer.
#[derive(Debug)]
pub enum ToolError {
    /// The session's rules refuse the call; the text tells the contestant what to do instead.
    Refused(String),
    /// Something on our side failed.
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for ToolError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

type ToolResult = Result<Reply, ToolError>;

/// A scene the rules accepted, with its level.
struct Accepted {
    level: Level,
    valid: ValidScene,
}

/// One contestant's session.
#[derive(Debug)]
pub struct Session {
    level: Option<Level>,
    arm: Arm,
    out: Option<PathBuf>,
    runs_used: u32,
    submitted: bool,
    seq: u32,
}

impl Session {
    /// A session on `level` (any level when `None`) under `arm`, recording into `out` when given.
    #[must_use]
    pub const fn new(level: Option<Level>, arm: Arm, out: Option<PathBuf>) -> Self {
        Self {
            level,
            arm,
            out,
            runs_used: 0,
            submitted: false,
            seq: 0,
        }
    }

    /// Whether `submit` has been accepted.
    #[cfg(test)]
    pub const fn submitted(&self) -> bool {
        self.submitted
    }

    /// The rules text, or the scene JSON Schema.
    ///
    /// # Errors
    /// When the schema cannot be serialized, which does not happen.
    pub fn rules(schema: bool) -> ToolResult {
        if schema {
            let text =
                serde_json::to_string_pretty(&scene::schema()).map_err(anyhow::Error::from)?;
            Ok(Reply::text(text))
        } else {
            Ok(Reply::text(rules_text::rules_text(&Rules::V1)))
        }
    }

    /// Every level, one per line.
    ///
    /// # Errors
    /// When an embedded level does not parse, which a test catches before release.
    pub fn levels() -> ToolResult {
        Ok(Reply::text(level_text::levels_text(&catalog::levels()?)))
    }

    /// One level in words and JSON, with a picture of it before anything is placed.
    ///
    /// # Errors
    /// Refused when a pilot session asks about another level.
    pub fn level(&self, id: &str) -> ToolResult {
        if let Some(assigned) = &self.level
            && assigned.id != id
        {
            return Err(ToolError::Refused(format!(
                "your level is {0}; call level with id \"{0}\"",
                assigned.id
            )));
        }
        let entry = catalog::find(id).map_err(|e| ToolError::Refused(e.to_string()))?;
        let level = catalog::level(entry)?;
        let empty = scene::Scene {
            level: level.id.clone(),
            parts: vec![],
        };
        let valid = clattertrap_core::validate::validate(&level, &empty, &Rules::V1)
            .map_err(|p| picture::problems_error(&p))?;
        let sim = Simulator::new(&level, &valid, &Rules::V1);
        Ok(Reply {
            text: level_text::level_text(&level)?,
            png: Some(picture::png_bytes(&level, &valid, &sim)?),
        })
    }

    /// Validates a scene: `ok: N parts placed`, or every problem, one per line.
    ///
    /// # Errors
    /// Refused after `submit`.
    pub fn check(&mut self, scene: &Value) -> ToolResult {
        self.open()?;
        let (text, problems) = match self.accept(scene) {
            Ok(accepted) => (ok_text(&accepted.valid), None),
            Err(problems) => (problems_text(&problems), Some(problem_strings(&problems))),
        };
        let seq = self.next_seq();
        self.record_call(&Call {
            seq,
            tool: Tool::Check,
            scene: scene.clone(),
            problems,
            outcome: None,
            name: None,
            prediction: None,
        })?;
        Ok(Reply::text(text))
    }

    /// A picture of a scene at the start of its run, or the problems.
    ///
    /// # Errors
    /// Refused after `submit`.
    pub fn look(&mut self, scene: &Value) -> ToolResult {
        self.open()?;
        let (reply, problems) = match self.accept(scene) {
            Ok(accepted) => {
                let sim = Simulator::new(&accepted.level, &accepted.valid, &Rules::V1);
                let png = picture::png_bytes(&accepted.level, &accepted.valid, &sim)?;
                (
                    Reply {
                        text: ok_text(&accepted.valid),
                        png: Some(png),
                    },
                    None,
                )
            }
            Err(problems) => (
                Reply::text(problems_text(&problems)),
                Some(problem_strings(&problems)),
            ),
        };
        let seq = self.next_seq();
        self.record_call(&Call {
            seq,
            tool: Tool::Look,
            scene: scene.clone(),
            problems,
            outcome: None,
            name: None,
            prediction: None,
        })?;
        Ok(reply)
    }

    /// Runs a scene to its end: the event trace, the outcome, the runs left, and the final frame.
    ///
    /// # Errors
    /// Refused after `submit` or once the arm's run budget is spent. Problems cost no budget.
    pub fn run(&mut self, scene: &Value) -> ToolResult {
        self.open()?;
        let budget = self.arm.run_budget();
        if self.runs_used >= budget {
            return Err(ToolError::Refused(if budget == 0 {
                "this arm allows 0 runs; use check and look, then submit".to_owned()
            } else {
                format!("this arm allows {budget} runs and you have used {budget}; submit")
            }));
        }
        let accepted = match self.accept(scene) {
            Ok(accepted) => accepted,
            Err(problems) => {
                let seq = self.next_seq();
                self.record_call(&Call {
                    seq,
                    tool: Tool::Run,
                    scene: scene.clone(),
                    problems: Some(problem_strings(&problems)),
                    outcome: None,
                    name: None,
                    prediction: None,
                })?;
                return Ok(Reply::text(problems_text(&problems)));
            }
        };
        self.runs_used += 1;
        let (outcome, events, sim) = run::run_to_end(&accepted.level, &accepted.valid);
        let mut text = String::new();
        for event in &events {
            text.push_str(&run::trace_line(*event, sim.names(), Rules::V1.dt_s));
            text.push('\n');
        }
        text.push_str(&outcome_text(&outcome)?);
        // Writing to a String cannot fail.
        let _ = writeln!(text, "runs left: {}", budget - self.runs_used);
        let png = picture::png_bytes(&accepted.level, &accepted.valid, &sim)?;
        let seq = self.next_seq();
        self.record_call(&Call {
            seq,
            tool: Tool::Run,
            scene: scene.clone(),
            problems: None,
            outcome: Some(outcome.clone()),
            name: None,
            prediction: None,
        })?;
        Ok(Reply {
            text,
            png: Some(png),
        })
    }

    /// Submits the machine: once, with a name and a one-sentence prediction. Ends the session.
    ///
    /// # Errors
    /// Refused after a submission, or when the name or prediction is empty or too long.
    pub fn submit(&mut self, scene: &Value, name: &str, prediction: &str) -> ToolResult {
        self.open()?;
        check_length("name", name, NAME_MAX_CHARS, "name the machine")?;
        check_length(
            "prediction",
            prediction,
            PREDICTION_MAX_CHARS,
            "predict in one sentence what the machine will do",
        )?;
        let accepted = match self.accept(scene) {
            Ok(accepted) => accepted,
            Err(problems) => {
                let seq = self.next_seq();
                self.record_call(&Call {
                    seq,
                    tool: Tool::Submit,
                    scene: scene.clone(),
                    problems: Some(problem_strings(&problems)),
                    outcome: None,
                    name: Some(name.to_owned()),
                    prediction: Some(prediction.to_owned()),
                })?;
                return Ok(Reply::text(problems_text(&problems)));
            }
        };
        let (outcome, _, _) = run::run_to_end(&accepted.level, &accepted.valid);
        let seq = self.next_seq();
        self.record_call(&Call {
            seq,
            tool: Tool::Submit,
            scene: scene.clone(),
            problems: None,
            outcome: Some(outcome.clone()),
            name: Some(name.to_owned()),
            prediction: Some(prediction.to_owned()),
        })?;
        if let Some(dir) = &self.out {
            write_submission(
                dir,
                &Submission {
                    level: accepted.level.id.clone(),
                    arm: self.arm,
                    name: name.to_owned(),
                    prediction: prediction.to_owned(),
                    scene: scene.clone(),
                    outcome: outcome.clone(),
                },
            )?;
        }
        self.submitted = true;
        let mut text = format!("Submitted {name:?}.\n");
        text.push_str(&outcome_text(&outcome)?);
        text.push_str("Your session is complete; there is nothing more to do.\n");
        Ok(Reply::text(text))
    }

    /// Refuses once the submission is in.
    fn open(&self) -> Result<(), ToolError> {
        if self.submitted {
            return Err(ToolError::Refused(
                "already submitted; the session is over".to_owned(),
            ));
        }
        Ok(())
    }

    /// Parses and validates a scene against its level: the assigned one, or the one it names.
    fn accept(&self, scene: &Value) -> Result<Accepted, Vec<Problem>> {
        let json = scene_json(scene);
        let level = match &self.level {
            Some(level) => level.clone(),
            None => match level_named(&json) {
                Ok(level) => level,
                Err(problem) => return Err(vec![problem]),
            },
        };
        check(&level, &json, &Rules::V1).map(|valid| Accepted { level, valid })
    }

    fn next_seq(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }

    fn record_call(&self, call: &Call) -> Result<(), ToolError> {
        if let Some(dir) = &self.out {
            write_call(dir, call)?;
        }
        Ok(())
    }
}

/// The level a scene names, when no level is assigned.
fn level_named(scene_json: &str) -> Result<Level, Problem> {
    let scene = clattertrap_core::validate::parse_scene(scene_json)?;
    let entry = catalog::find(&scene.level).map_err(|_| Problem {
        path: "level".to_owned(),
        message: format!("no level {:?}; call levels to see the ids", scene.level),
    })?;
    catalog::level(entry).map_err(|e| Problem {
        path: "level".to_owned(),
        message: e.to_string(),
    })
}

fn check_length(field: &str, value: &str, max: usize, fix: &str) -> Result<(), ToolError> {
    let count = value.chars().count();
    if value.trim().is_empty() {
        return Err(ToolError::Refused(format!("{field}: is empty; {fix}")));
    }
    if count > max {
        return Err(ToolError::Refused(format!(
            "{field}: {count} characters is over the {max} allowed; shorten it"
        )));
    }
    Ok(())
}

fn ok_text(valid: &ValidScene) -> String {
    let n = valid.scene_part_count();
    format!("ok: {n} part{} placed", if n == 1 { "" } else { "s" })
}

fn problem_strings(problems: &[Problem]) -> Vec<String> {
    problems.iter().map(ToString::to_string).collect()
}

fn problems_text(problems: &[Problem]) -> String {
    problem_strings(problems).join("\n")
}

fn outcome_text(outcome: &Outcome) -> Result<String> {
    let mut text = serde_json::to_string_pretty(outcome)?;
    text.push('\n');
    Ok(text)
}
