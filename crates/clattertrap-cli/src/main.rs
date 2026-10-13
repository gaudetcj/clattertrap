//! The `clattertrap` command line. Subcommands arrive with their milestones.
#![forbid(unsafe_code)]

use clap::Parser;

/// Build Rube Goldberg machines with AI models and watch whether they work.
#[derive(Parser)]
#[command(name = "clattertrap", bin_name = "clattertrap", version)]
struct Cli {}

fn main() {
    let Cli {} = Cli::parse();
    println!("rules v{}", clattertrap_core::rules::RULES_VERSION);
}
