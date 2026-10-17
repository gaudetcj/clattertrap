//! The `clattertrap` command line: `rules`, `levels`, `level`, `check`, `look`, `run`, `mcp`, `arena`,
//! and `report`.
#![forbid(unsafe_code)]

mod arena;
mod catalog;
mod check;
mod level_text;
mod mcp;
mod picture;
mod report;
mod rules_text;
mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::sim::Simulator;
use clattertrap_core::{level, scene};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Build Rube Goldberg machines with AI models and watch whether they work.
#[derive(Parser)]
#[command(name = "clattertrap", bin_name = "clattertrap", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the rules contestants play by, or a JSON Schema
    Rules {
        /// Print the scene JSON Schema instead
        #[arg(long, conflicts_with = "level_schema")]
        schema: bool,
        /// Print the level JSON Schema instead
        #[arg(long)]
        level_schema: bool,
    },
    /// List every level with its tier and goal
    Levels,
    /// Describe one level in words and JSON
    Level {
        /// Level id, as `levels` lists it
        id: String,
        /// Also write a PNG of the level at the start
        #[arg(long, value_name = "PATH")]
        png: Option<PathBuf>,
    },
    /// Validate a scene file against its level; exits 1 with the problems
    Check {
        /// Scene JSON file
        scene: PathBuf,
    },
    /// Write a PNG of a scene at the start of its run
    Look {
        /// Scene JSON file
        scene: PathBuf,
        /// Where to write; the default is the scene's path with a .png extension
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Run a scene to its end and print the outcome as JSON
    Run {
        /// Scene JSON file
        scene: PathBuf,
        /// Print every event, one per line, before the outcome
        #[arg(long)]
        trace: bool,
        /// Also write a PNG of the final frame
        #[arg(long, value_name = "PATH")]
        png: Option<PathBuf>,
    },
    /// Run a Claude model through the contestant tools and save every run (the pilot)
    Arena {
        /// Model alias or id; Claude models only
        #[arg(long)]
        model: String,
        /// Sessions per level
        #[arg(long, default_value_t = 1)]
        runs: u32,
        /// The arm, which sets the run budget
        #[arg(long, value_enum)]
        arm: mcp::arm::Arm,
        /// One level instead of all of them
        #[arg(long, value_name = "ID")]
        level: Option<String>,
        /// Where run directories go
        #[arg(long, value_name = "DIR", default_value = "pilot/runs")]
        out: PathBuf,
        /// The claude executable
        #[arg(long, default_value = "claude")]
        claude: String,
    },
    /// Summarize a directory of pilot runs: levels solved per model and arm, attempts, and exclusions
    Report {
        /// The runs directory, <model>/<level>/<arm>-NN underneath
        dir: PathBuf,
        /// One CSV row per run instead of the tables
        #[arg(long, conflicts_with = "sample")]
        csv: bool,
        /// Print this many counted submissions to watch, chosen with --seed
        #[arg(long, value_name = "N")]
        sample: Option<usize>,
        /// Seed for --sample
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    /// Serve the contestant tools over stdio as an MCP server
    Mcp {
        /// Assign one level; other levels are refused
        #[arg(long, value_name = "ID")]
        level: Option<String>,
        /// The arm, which sets the run budget
        #[arg(long, value_enum, default_value_t = mcp::arm::Arm::Workshop)]
        arm: mcp::arm::Arm,
        /// Directory for the call records and the submission
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
    },
}

/// Exit code for a scene the rules reject.
const REJECTED: u8 = 1;

fn main() -> ExitCode {
    match dispatch() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn dispatch() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::Rules {
            schema,
            level_schema,
        } => rules(schema, level_schema),
        Command::Levels => levels(),
        Command::Level { id, png } => level_command(&id, png.as_deref()),
        Command::Check { scene } => check_command(&scene),
        Command::Look { scene, out } => look(&scene, out),
        Command::Run { scene, trace, png } => run_command(&scene, trace, png.as_deref()),
        Command::Arena {
            model,
            runs,
            arm,
            level,
            out,
            claude,
        } => {
            arena::run(&arena::Options {
                model,
                runs,
                arm,
                level,
                out,
                claude,
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Report {
            dir,
            csv,
            sample,
            seed,
        } => {
            report::run(&report::Options {
                dir,
                csv,
                sample,
                seed,
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp { level, arm, out } => {
            mcp::serve(level.as_deref(), arm, out)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn rules(schema: bool, level_schema: bool) -> Result<ExitCode> {
    if schema {
        println!("{}", serde_json::to_string_pretty(&scene::schema())?);
    } else if level_schema {
        println!("{}", serde_json::to_string_pretty(&level::schema())?);
    } else {
        print!("{}", rules_text::rules_text(&Rules::V1));
    }
    Ok(ExitCode::SUCCESS)
}

fn levels() -> Result<ExitCode> {
    print!("{}", level_text::levels_text(&catalog::levels()?));
    Ok(ExitCode::SUCCESS)
}

fn level_command(id: &str, png: Option<&Path>) -> Result<ExitCode> {
    let level = catalog::level(catalog::find(id)?)?;
    print!("{}", level_text::level_text(&level)?);
    if let Some(path) = png {
        let empty = Scene {
            level: level.id.clone(),
            parts: vec![],
        };
        picture::write_start_png(&level, &empty, path)?;
        println!("wrote {}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn check_command(scene: &Path) -> Result<ExitCode> {
    let Some(loaded) = check::load_or_report(scene)? else {
        return Ok(ExitCode::from(REJECTED));
    };
    let n = loaded.valid.scene_part_count();
    println!("ok: {n} part{} placed", if n == 1 { "" } else { "s" });
    Ok(ExitCode::SUCCESS)
}

fn look(scene: &Path, out: Option<PathBuf>) -> Result<ExitCode> {
    let Some(loaded) = check::load_or_report(scene)? else {
        return Ok(ExitCode::from(REJECTED));
    };
    let out = out.unwrap_or_else(|| scene.with_extension("png"));
    let sim = Simulator::new(&loaded.level, &loaded.valid, &Rules::V1);
    picture::write_png(&loaded.level, &loaded.valid, &sim, &out)?;
    println!("wrote {}", out.display());
    Ok(ExitCode::SUCCESS)
}

fn run_command(scene: &Path, trace: bool, png: Option<&Path>) -> Result<ExitCode> {
    let Some(loaded) = check::load_or_report(scene)? else {
        return Ok(ExitCode::from(REJECTED));
    };
    let (outcome, events, sim) = run::run_to_end(&loaded.level, &loaded.valid);
    if trace {
        for event in &events {
            println!("{}", run::trace_line(*event, sim.names(), Rules::V1.dt_s));
        }
    }
    println!("{}", serde_json::to_string_pretty(&outcome)?);
    if let Some(path) = png {
        picture::write_png(&loaded.level, &loaded.valid, &sim, path)?;
    }
    Ok(ExitCode::SUCCESS)
}
