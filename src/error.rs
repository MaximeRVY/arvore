#[derive(Debug, thiserror::Error)]
pub enum ArvoreError {
    #[error("not inside a git repository")]
    NotARepo,
    #[error("worktree '{0}' not found")]
    WorktreeNotFound(String),
    #[error("worktree '{0}' has uncommitted changes (use --force to remove)")]
    DirtyWorktree(String),
    #[error("git command failed: {0}")]
    GitError(String),
    #[error("config error: {0}")]
    ConfigError(String),
}
