use anyhow::Result;
use colored::Colorize;
use dialoguer::MultiSelect;

use crate::git;

struct CleanCandidate {
    branch: String,
    path: std::path::PathBuf,
    merged: bool,
    remote_deleted: bool,
    dirty: bool,
}

impl CleanCandidate {
    fn reason(&self) -> String {
        let mut reasons = Vec::new();
        if self.merged {
            reasons.push("merged");
        }
        if self.remote_deleted {
            reasons.push("remote deleted");
        }
        reasons.join(" + ")
    }
}

pub fn run(dry_run: bool) -> Result<()> {
    git::ensure_repo()?;

    println!("{}", "Fetching and pruning remotes...".cyan());
    git::fetch_prune()?;

    let main = git::main_branch()?;
    let merged = git::merged_branches(&main)?;
    let worktrees = git::worktree_list()?;

    let mut candidates: Vec<CleanCandidate> = Vec::new();

    for wt in &worktrees {
        if wt.is_bare {
            continue;
        }

        let branch = match &wt.branch {
            Some(b) => b.clone(),
            None => continue,
        };

        if branch == main {
            continue;
        }

        let is_merged = merged.contains(&branch);
        let remote_deleted = !git::remote_branch_exists(&branch)?;

        if !is_merged && !remote_deleted {
            continue;
        }

        let dirty = if wt.path.exists() {
            git::is_dirty(&wt.path).unwrap_or(false)
        } else {
            false
        };

        candidates.push(CleanCandidate {
            branch,
            path: wt.path.clone(),
            merged: is_merged,
            remote_deleted,
            dirty,
        });
    }

    if candidates.is_empty() {
        println!("{}", "No worktrees to clean up.".green());
        return Ok(());
    }

    println!(
        "\n{} candidate(s) for cleanup:\n",
        candidates.len().to_string().bold()
    );

    for (i, c) in candidates.iter().enumerate() {
        let dirty_warn = if c.dirty {
            " ⚠ dirty".red().to_string()
        } else {
            String::new()
        };
        println!(
            "  {}. {} ({}){}\n     {}",
            i + 1,
            c.branch.yellow().bold(),
            c.reason().dimmed(),
            dirty_warn,
            c.path.display().to_string().dimmed()
        );
    }
    println!();

    if dry_run {
        println!("{}", "Dry run - no worktrees removed.".cyan());
        return Ok(());
    }

    let labels: Vec<String> = candidates
        .iter()
        .map(|c| {
            let dirty_mark = if c.dirty { " ⚠ dirty" } else { "" };
            format!("{} ({}){}", c.branch, c.reason(), dirty_mark)
        })
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select worktrees to remove")
        .items(&labels)
        .interact()?;

    if selections.is_empty() {
        println!("{}", "Nothing selected.".yellow());
        return Ok(());
    }

    for idx in selections {
        let c = &candidates[idx];
        let force = c.dirty;
        match git::worktree_remove(&c.path, force) {
            Ok(()) => println!("{} Removed {}", "✓".green().bold(), c.branch.cyan()),
            Err(e) => eprintln!(
                "{} Failed to remove {}: {}",
                "✗".red().bold(),
                c.branch.yellow(),
                e
            ),
        }
    }

    git::worktree_prune()?;
    println!("\n{}", "Cleanup complete.".green().bold());

    Ok(())
}
