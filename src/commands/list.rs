use anyhow::Result;
use colored::Colorize;

use crate::git;

pub fn run(porcelain: bool) -> Result<()> {
    git::ensure_repo()?;

    let worktrees = git::worktree_list()?;

    if porcelain {
        for wt in &worktrees {
            let branch_name = wt.branch.as_deref().unwrap_or("(detached)");
            let dirty = if !wt.is_bare && wt.path.exists() {
                git::is_dirty(&wt.path).unwrap_or(false)
            } else {
                false
            };
            let short_head = &wt.head[..wt.head.len().min(8)];
            println!(
                "{}\t{}\t{}\t{}",
                branch_name,
                wt.path.display(),
                if dirty { "dirty" } else { "clean" },
                short_head
            );
        }
        return Ok(());
    }

    if worktrees.is_empty() {
        println!("{}", "No worktrees found.".yellow());
        return Ok(());
    }

    for wt in &worktrees {
        let branch_name = wt.branch.as_deref().unwrap_or("(detached)");
        let dirty = if !wt.is_bare && wt.path.exists() {
            git::is_dirty(&wt.path).unwrap_or(false)
        } else {
            false
        };
        let short_head = &wt.head[..wt.head.len().min(8)];

        let branch_display = if dirty {
            branch_name.yellow().bold().to_string()
        } else {
            branch_name.green().bold().to_string()
        };

        let dirty_indicator = if dirty {
            " [modified]".red().to_string()
        } else {
            String::new()
        };

        println!(
            "  {} {} {}{}",
            short_head.dimmed(),
            branch_display,
            wt.path.display().to_string().dimmed(),
            dirty_indicator
        );
    }

    Ok(())
}
