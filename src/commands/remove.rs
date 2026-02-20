use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use colored::Colorize;

use crate::config::Config;
use crate::error::ArvoreError;
use crate::git;

pub fn run(config: &Config, target: &str, force: bool) -> Result<()> {
    git::ensure_repo()?;

    let worktree_path = resolve_target(config, target)?;

    let worktrees = git::worktree_list()?;
    let found = worktrees.iter().any(|wt| wt.path == worktree_path);
    if !found {
        bail!(ArvoreError::WorktreeNotFound(target.to_string()));
    }

    if !force && worktree_path.exists() && git::is_dirty(&worktree_path)? {
        bail!(ArvoreError::DirtyWorktree(target.to_string()));
    }

    git::worktree_remove(&worktree_path, force)?;
    git::worktree_prune()?;

    cleanup_empty_parents(&worktree_path);

    println!("{} Removed worktree {}", "✓".green().bold(), target.cyan());

    Ok(())
}

fn resolve_target(config: &Config, target: &str) -> Result<PathBuf> {
    let as_path = PathBuf::from(target);
    if as_path.is_absolute() && as_path.exists() {
        return Ok(as_path);
    }

    let repo_name = git::repo_name()?;
    Ok(config.worktree_path(&repo_name, target))
}

fn cleanup_empty_parents(path: &Path) {
    if let Some(parent) = path.parent()
        && parent.exists()
        && let Ok(entries) = std::fs::read_dir(parent)
        && entries.count() == 0
    {
        let _ = std::fs::remove_dir(parent);
    }
}
