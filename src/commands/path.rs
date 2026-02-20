use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git;

pub fn run(config: &Config, branch: &str) -> Result<()> {
    git::ensure_repo()?;

    let repo_name = git::repo_name()?;
    let worktree_path = config.worktree_path(&repo_name, branch);

    if !worktree_path.exists() {
        eprintln!(
            "{} worktree path does not exist yet: {}",
            "warning:".yellow().bold(),
            worktree_path.display()
        );
    }

    println!("{}", worktree_path.display());

    Ok(())
}
