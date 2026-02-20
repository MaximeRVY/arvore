use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Result};

use crate::error::ArvoreError;

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head: String,
    pub is_bare: bool,
}

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| ArvoreError::GitError(format!("failed to execute git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(ArvoreError::GitError(stderr.trim().to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_git_in(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| ArvoreError::GitError(format!("failed to execute git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(ArvoreError::GitError(stderr.trim().to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn repo_root() -> Result<PathBuf> {
    let out = run_git(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(out))
}

pub fn repo_name() -> Result<String> {
    let root = repo_root()?;
    let name = root
        .file_name()
        .ok_or(ArvoreError::GitError("cannot determine repo name".into()))?
        .to_string_lossy()
        .to_string();
    Ok(name)
}

pub fn is_inside_worktree() -> Result<bool> {
    match run_git(&["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) => Ok(out == "true"),
        Err(_) => Ok(false),
    }
}

pub fn parse_worktree_porcelain(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut path: Option<PathBuf> = None;
    let mut head = String::new();
    let mut branch: Option<String> = None;
    let mut is_bare = false;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(p) = path.take() {
                worktrees.push(WorktreeInfo {
                    path: p,
                    branch: branch.take(),
                    head: head.clone(),
                    is_bare,
                });
                head.clear();
                is_bare = false;
            }
            continue;
        }

        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            head = h.to_string();
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            branch = Some(b.to_string());
        } else if line == "bare" {
            is_bare = true;
        }
    }

    if let Some(p) = path.take() {
        worktrees.push(WorktreeInfo {
            path: p,
            branch: branch.take(),
            head,
            is_bare,
        });
    }

    worktrees
}

pub fn worktree_list() -> Result<Vec<WorktreeInfo>> {
    let out = run_git(&["worktree", "list", "--porcelain"])?;
    Ok(parse_worktree_porcelain(&out))
}

pub fn worktree_add(
    path: &Path,
    branch_arg: &str,
    new_branch: bool,
    base: Option<&str>,
) -> Result<()> {
    let path_str = path.to_string_lossy();
    let mut args = vec!["worktree", "add"];

    if new_branch {
        args.push(&path_str);
        args.push("-b");
        args.push(branch_arg);
        if let Some(b) = base {
            args.push(b);
        }
    } else {
        args.push(&path_str);
        args.push(branch_arg);
    }

    run_git(&args)?;
    Ok(())
}

pub fn worktree_remove(path: &Path, force: bool) -> Result<()> {
    let path_str = path.to_string_lossy();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_str);
    run_git(&args)?;
    Ok(())
}

pub fn worktree_prune() -> Result<()> {
    run_git(&["worktree", "prune"])?;
    Ok(())
}

pub fn status_porcelain(path: &Path) -> Result<String> {
    run_git_in(path, &["status", "--porcelain"])
}

pub fn is_dirty(path: &Path) -> Result<bool> {
    let status = status_porcelain(path)?;
    Ok(!status.is_empty())
}

pub fn fetch_prune() -> Result<()> {
    run_git(&["fetch", "--prune"])?;
    Ok(())
}

pub fn merged_branches(main_branch: &str) -> Result<Vec<String>> {
    let out = run_git(&["branch", "--merged", main_branch])?;
    let branches = out
        .lines()
        .map(|l| l.trim().trim_start_matches("* ").to_string())
        .filter(|b| !b.is_empty() && b != main_branch)
        .collect();
    Ok(branches)
}

pub fn remote_branch_exists(branch: &str) -> Result<bool> {
    let out = run_git(&["ls-remote", "--heads", "origin", branch])?;
    Ok(!out.is_empty())
}

#[allow(dead_code)]
pub fn current_branch() -> Result<String> {
    run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
}

pub fn main_branch() -> Result<String> {
    if run_git(&["rev-parse", "--verify", "refs/heads/main"]).is_ok() {
        return Ok("main".to_string());
    }
    if run_git(&["rev-parse", "--verify", "refs/heads/master"]).is_ok() {
        return Ok("master".to_string());
    }
    if run_git(&["rev-parse", "--verify", "refs/remotes/origin/main"]).is_ok() {
        return Ok("main".to_string());
    }
    if run_git(&["rev-parse", "--verify", "refs/remotes/origin/master"]).is_ok() {
        return Ok("master".to_string());
    }
    bail!(ArvoreError::GitError(
        "cannot detect main branch (tried main, master)".into()
    ));
}

pub fn ensure_repo() -> Result<()> {
    if !is_inside_worktree()? {
        bail!(ArvoreError::NotARepo);
    }
    Ok(())
}

pub fn branch_exists_locally(branch: &str) -> Result<bool> {
    match run_git(&["rev-parse", "--verify", &format!("refs/heads/{branch}")]) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_worktree() {
        let output = "\
worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main

";
        let wts = parse_worktree_porcelain(output);
        assert_eq!(wts.len(), 1);
        assert_eq!(wts[0].path, PathBuf::from("/path/to/repo"));
        assert_eq!(wts[0].head, "abc123def456");
        assert_eq!(wts[0].branch.as_deref(), Some("main"));
        assert!(!wts[0].is_bare);
    }

    #[test]
    fn parse_multiple_worktrees() {
        let output = "\
worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main

worktree /path/to/wt1
HEAD def456abc789
branch refs/heads/feature-x

";
        let wts = parse_worktree_porcelain(output);
        assert_eq!(wts.len(), 2);
        assert_eq!(wts[0].path, PathBuf::from("/path/to/repo"));
        assert_eq!(wts[0].branch.as_deref(), Some("main"));
        assert_eq!(wts[1].path, PathBuf::from("/path/to/wt1"));
        assert_eq!(wts[1].head, "def456abc789");
        assert_eq!(wts[1].branch.as_deref(), Some("feature-x"));
    }

    #[test]
    fn parse_bare_worktree() {
        let output = "\
worktree /path/to/repo
HEAD abc123def456
bare

";
        let wts = parse_worktree_porcelain(output);
        assert_eq!(wts.len(), 1);
        assert!(wts[0].is_bare);
        assert!(wts[0].branch.is_none());
    }

    #[test]
    fn parse_detached_head() {
        let output = "\
worktree /path/to/repo
HEAD abc123def456
detached

";
        let wts = parse_worktree_porcelain(output);
        assert_eq!(wts.len(), 1);
        assert!(wts[0].branch.is_none());
        assert!(!wts[0].is_bare);
    }

    #[test]
    fn parse_empty_output() {
        let wts = parse_worktree_porcelain("");
        assert!(wts.is_empty());
    }

    #[test]
    fn parse_no_trailing_newline() {
        let output = "\
worktree /path/to/repo
HEAD abc123def456
branch refs/heads/main";
        let wts = parse_worktree_porcelain(output);
        assert_eq!(wts.len(), 1);
        assert_eq!(wts[0].path, PathBuf::from("/path/to/repo"));
        assert_eq!(wts[0].branch.as_deref(), Some("main"));
    }
}
