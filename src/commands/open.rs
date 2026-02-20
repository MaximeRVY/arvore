use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use colored::Colorize;

use crate::config::Config;
use crate::error::ArvoreError;
use crate::git;

pub fn run(config: &Config, branch: &str, cursor: bool, warp: bool, all: bool) -> Result<()> {
    git::ensure_repo()?;

    let repo_name = git::repo_name()?;
    let worktree_path = config.worktree_path(&repo_name, branch);

    if !worktree_path.exists() {
        bail!(ArvoreError::WorktreeNotFound(branch.to_string()));
    }

    let open_warp = all || warp || !cursor;
    let open_cursor = all || cursor || !warp;

    open_path(&worktree_path, open_warp, open_cursor)?;

    Ok(())
}

pub fn open_path(path: &Path, warp: bool, cursor: bool) -> Result<()> {
    if warp {
        Command::new("open")
            .args(["-a", "Warp"])
            .arg(path)
            .spawn()
            .map_err(|e| ArvoreError::GitError(format!("failed to open Warp: {e}")))?;
        println!("{} Opened in {}", "✓".green().bold(), "Warp".cyan());
    }

    if cursor {
        Command::new("cursor")
            .arg(path)
            .spawn()
            .map_err(|e| ArvoreError::GitError(format!("failed to open Cursor: {e}")))?;
        println!("{} Opened in {}", "✓".green().bold(), "Cursor".cyan());
    }

    Ok(())
}
