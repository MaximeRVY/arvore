use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git;

pub fn run(config: &Config, branch: &str, from: Option<&str>, open: bool) -> Result<()> {
    git::ensure_repo()?;

    let repo_name = git::repo_name()?;
    let worktree_path = config.worktree_path(&repo_name, branch);

    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match from {
        Some(base_ref) => {
            git::worktree_add(&worktree_path, branch, true, Some(base_ref))?;
        }
        None => {
            let exists_locally = git::branch_exists_locally(branch)?;
            let exists_remotely = git::remote_branch_exists(branch)?;

            if exists_locally || exists_remotely {
                git::worktree_add(&worktree_path, branch, false, None)?;
            } else {
                git::worktree_add(&worktree_path, branch, true, None)?;
            }
        }
    }

    println!(
        "{} Created worktree at {}",
        "✓".green().bold(),
        worktree_path.display().to_string().cyan()
    );

    if open {
        crate::commands::open::open_path(&worktree_path, true, true)?;
    }

    Ok(())
}
