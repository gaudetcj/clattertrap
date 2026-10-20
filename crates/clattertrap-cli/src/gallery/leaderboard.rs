//! `clattertrap leaderboard`: levels solved per model and arm over a gallery or runs directory,
//! ranked, as a text table or as the Markdown block the README carries.

use crate::catalog::{self, is_hard};
use crate::mcp::arm::Arm;
use crate::report::{Run, load, mean};
use anyhow::{Result, ensure};
use clattertrap_core::rules::RULES_VERSION;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::PathBuf;

/// What one `leaderboard` invocation prints.
#[derive(Debug)]
pub struct Options {
    /// A runs or gallery directory: `summary.json` three levels down.
    pub dir: PathBuf,
    /// The Markdown table and count line instead of the text table.
    pub markdown: bool,
}

/// Levels solved and attempted under one arm; a level counts once however many runs played it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArmCount {
    /// Levels met by at least one counted submission.
    pub solved: usize,
    /// Levels with at least one counted run.
    pub attempted: usize,
    /// Solved levels of tier 4 or 5.
    pub hard_solved: usize,
    /// Attempted levels of tier 4 or 5.
    pub hard_attempted: usize,
}

impl ArmCount {
    fn cell(self) -> String {
        format!("{}/{}", self.solved, self.attempted)
    }

    fn hard_cell(self) -> String {
        format!("{}/{}", self.hard_solved, self.hard_attempted)
    }
}

/// One model's row.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The model id as the summaries name it.
    pub model: String,
    /// The blueprint arm.
    pub blueprint: ArmCount,
    /// The workshop arm.
    pub workshop: ArmCount,
    /// Mean cost over the counted runs that reported one.
    pub mean_usd: f64,
    /// Counted runs.
    pub runs: usize,
    /// Counted runs that ended with a submission, each a machine in the gallery.
    pub machines: usize,
}

/// Prints the leaderboard.
///
/// # Errors
/// When the directory holds no runs or a record cannot be read.
pub fn run(opts: &Options) -> Result<()> {
    let runs = load(&opts.dir)?;
    ensure!(
        !runs.is_empty(),
        "no runs under {}: expected <a>/<b>/<c>/summary.json three levels down",
        opts.dir.display()
    );
    let tiers = catalog::levels()?
        .into_iter()
        .map(|l| (l.id, l.tier))
        .collect::<Vec<_>>();
    let rows = rows(&runs, &tiers);
    let out = if opts.markdown {
        markdown(&rows, tiers.len())
    } else {
        text(&rows)
    };
    print!("{out}");
    Ok(())
}

/// One row per model with a counted run, ranked: blueprint levels solved, then workshop levels
/// solved, mean cost, and id.
#[must_use]
pub fn rows(runs: &[Run], tiers: &[(String, u8)]) -> Vec<Row> {
    let counted: Vec<&Run> = runs.iter().filter(|r| r.counts()).collect();
    let models: BTreeSet<&str> = counted.iter().map(|r| r.summary.model.as_str()).collect();
    let mut rows: Vec<Row> = models
        .into_iter()
        .map(|model| {
            let mine: Vec<&Run> = counted
                .iter()
                .copied()
                .filter(|r| r.summary.model == model)
                .collect();
            Row {
                model: model.to_owned(),
                blueprint: arm_count(&mine, Arm::Blueprint, tiers),
                workshop: arm_count(&mine, Arm::Workshop, tiers),
                mean_usd: mean(mine.iter().filter_map(|r| r.summary.cost_usd)),
                runs: mine.len(),
                machines: mine.iter().filter(|r| r.submission.is_some()).count(),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.blueprint
            .solved
            .cmp(&a.blueprint.solved)
            .then(b.workshop.solved.cmp(&a.workshop.solved))
            .then(a.mean_usd.total_cmp(&b.mean_usd))
            .then(a.model.cmp(&b.model))
    });
    rows
}

fn arm_count(runs: &[&Run], arm: Arm, tiers: &[(String, u8)]) -> ArmCount {
    let mine = runs.iter().filter(|r| r.summary.arm == arm);
    let attempted: BTreeSet<&str> = mine.clone().map(|r| r.summary.level.as_str()).collect();
    let solved: BTreeSet<&str> = mine
        .filter(|r| r.solved())
        .map(|r| r.summary.level.as_str())
        .collect();
    let hard = |ids: &BTreeSet<&str>| ids.iter().filter(|id| is_hard(id, tiers)).count();
    ArmCount {
        solved: solved.len(),
        attempted: attempted.len(),
        hard_solved: hard(&solved),
        hard_attempted: hard(&attempted),
    }
}

/// The text table.
#[must_use]
pub fn text(rows: &[Row]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<17} blueprint  workshop  hard blueprint  hard workshop  mean USD",
        "model"
    );
    for r in rows {
        let _ = writeln!(
            out,
            "{:<17} {:>9} {:>9} {:>14} {:>13} {:>9.2}",
            r.model,
            r.blueprint.cell(),
            r.workshop.cell(),
            r.blueprint.hard_cell(),
            r.workshop.hard_cell(),
            r.mean_usd
        );
    }
    out
}

/// The Markdown table and the count line under it.
#[must_use]
pub fn markdown(rows: &[Row], total: usize) -> String {
    let mut out = String::from(
        "| model | blueprint | workshop | hard blueprint | hard workshop | mean USD |\n|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {:.2} |",
            r.model,
            r.blueprint.cell(),
            r.workshop.cell(),
            r.blueprint.hard_cell(),
            r.workshop.hard_cell(),
            r.mean_usd
        );
    }
    let runs: usize = rows.iter().map(|r| r.runs).sum();
    let machines: usize = rows.iter().map(|r| r.machines).sum();
    let _ = write!(
        out,
        "\n{runs} counted runs by {} models on rules version {RULES_VERSION}, {machines} of them with a machine and {} without a submission. A level counts once however many runs solved it, over the levels the model attempted in that arm, of {total}; hard levels are tiers 4 and 5.\n",
        rows.len(),
        runs - machines
    );
    out
}

#[cfg(test)]
mod tests {
    use super::{Row, rows};
    use crate::mcp::arm::Arm;
    use crate::records::{Status, Summary};
    use crate::report::{Calls, Run, Submitted};

    fn tiers() -> Vec<(String, u8)> {
        (1..=15)
            .map(|n| (format!("level-{n:02}"), if n > 12 { 4 } else { 1 }))
            .collect()
    }

    fn run(model: &str, level: &str, arm: Arm, status: Status, solved: bool, cost: f64) -> Run {
        Run {
            rel: format!("{model}/{level}/{arm}-01"),
            summary: Summary {
                model: model.to_owned(),
                level: level.to_owned(),
                arm,
                status,
                started_unix_s: 0,
                duration_s: 1.0,
                cost_usd: Some(cost),
                turns: None,
                tool_calls: 0,
                foreign_tools: vec![],
                result_subtype: None,
                exit_code: None,
                error: None,
            },
            submission: (status == Status::Submitted).then(|| Submitted {
                goal_met: solved,
                name: String::new(),
            }),
            calls: Calls::default(),
        }
    }

    /// Every level under both arms, solving the first `solved` blueprint levels.
    fn full_model(model: &str, solved: usize, cost: f64) -> Vec<Run> {
        let mut out = Vec::new();
        for (i, (level, _)) in tiers().into_iter().enumerate() {
            for arm in [Arm::Blueprint, Arm::Workshop] {
                let hit = arm == Arm::Blueprint && i < solved;
                out.push(run(model, &level, arm, Status::Submitted, hit, cost));
            }
        }
        out
    }

    #[test]
    fn blueprint_solved_ranks_first_whatever_the_coverage_and_cost_breaks_ties() {
        let mut runs = full_model("full-pricey", 1, 2.0);
        runs.extend(full_model("full-cheap", 1, 0.5));
        runs.push(run(
            "partial",
            "level-13",
            Arm::Blueprint,
            Status::Submitted,
            true,
            0.1,
        ));
        runs.push(run(
            "partial",
            "level-14",
            Arm::Blueprint,
            Status::Submitted,
            true,
            0.1,
        ));
        let rows = rows(&runs, &tiers());
        let order: Vec<&str> = rows.iter().map(|r| r.model.as_str()).collect();
        assert_eq!(order, ["partial", "full-cheap", "full-pricey"]);
        let partial: &Row = &rows[0];
        assert_eq!(partial.blueprint.solved, 2);
        assert_eq!(partial.blueprint.hard_solved, 2);
        assert_eq!(partial.workshop.attempted, 0);
    }

    #[test]
    fn a_level_counts_once_and_runs_that_do_not_count_are_left_out() {
        let runs = vec![
            run("m", "level-01", Arm::Workshop, Status::Submitted, true, 1.0),
            run(
                "m",
                "level-01",
                Arm::Workshop,
                Status::Submitted,
                false,
                1.0,
            ),
            run(
                "m",
                "level-02",
                Arm::Workshop,
                Status::NoSubmission,
                false,
                1.0,
            ),
            run("m", "level-03", Arm::Workshop, Status::Aborted, false, 9.0),
            run("m", "level-04", Arm::Workshop, Status::Flagged, true, 9.0),
        ];
        let rows = rows(&runs, &tiers());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].workshop.solved, 1);
        assert_eq!(rows[0].workshop.attempted, 2);
        assert_eq!(rows[0].runs, 3);
        assert!((rows[0].mean_usd - 1.0).abs() < 1e-9);
    }
}
