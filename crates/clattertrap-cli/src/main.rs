//! The `clattertrap` command line: what contestants and people read and run.
#![forbid(unsafe_code)]

mod catalog;
mod level_text;
mod picture;
mod rules_text;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Scene;
use clattertrap_core::{level, scene};
use std::path::PathBuf;
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
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Cli::parse().command {
        Command::Rules {
            schema,
            level_schema,
        } => {
            if schema {
                println!("{}", serde_json::to_string_pretty(&scene::schema())?);
            } else if level_schema {
                println!("{}", serde_json::to_string_pretty(&level::schema())?);
            } else {
                print!("{}", rules_text::rules_text(&Rules::V1));
            }
        }
        Command::Levels => {
            for level in catalog::levels()? {
                println!(
                    "{:<14} {}  {:<6} {}",
                    level.id,
                    level.tier,
                    level_text::goal_kind(&level.goal),
                    level.title
                );
            }
        }
        Command::Level { id, png } => {
            let level = catalog::level(catalog::find(&id)?)?;
            print!("{}", level_text::level_text(&level)?);
            if let Some(path) = png {
                let empty = Scene {
                    level: level.id.clone(),
                    parts: vec![],
                };
                picture::write_start_png(&level, &empty, &path)?;
                println!("wrote {}", path.display());
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
