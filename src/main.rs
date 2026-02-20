mod cli;
mod commands;
mod config;
mod error;
mod git;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};
use config::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load(cli.config.as_deref())?;

    match &cli.command {
        Commands::Create { branch, from, open } => {
            commands::create::run(&config, branch, from.as_deref(), *open)?;
        }
        Commands::List { porcelain } => {
            commands::list::run(*porcelain)?;
        }
        Commands::Remove { target, force } => {
            commands::remove::run(&config, target, *force)?;
        }
        Commands::Open {
            branch,
            cursor,
            warp,
            all,
        } => {
            commands::open::run(&config, branch, *cursor, *warp, *all)?;
        }
        Commands::Path { branch } => {
            commands::path::run(&config, branch)?;
        }
        Commands::Clean { dry_run } => {
            commands::clean::run(*dry_run)?;
        }
        Commands::Completions { shell } => {
            commands::completions::run(shell)?;
        }
    }

    Ok(())
}
