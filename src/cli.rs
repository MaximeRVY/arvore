use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "arvore", about = "A fast git worktree manager", version)]
pub struct Cli {
    #[arg(long, global = true, help = "Path to config file")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Create a new worktree")]
    Create {
        branch: String,
        #[arg(long, help = "Base ref to branch from")]
        from: Option<String>,
        #[arg(long, help = "Open worktree after creation")]
        open: bool,
    },

    #[command(name = "ls", about = "List all worktrees")]
    List {
        #[arg(long, help = "Machine-readable output")]
        porcelain: bool,
    },

    #[command(name = "rm", about = "Remove a worktree")]
    Remove {
        target: String,
        #[arg(long, help = "Force removal even if dirty")]
        force: bool,
    },

    #[command(about = "Open a worktree in editor/terminal")]
    Open {
        branch: String,
        #[arg(long, help = "Open in Cursor")]
        cursor: bool,
        #[arg(long, help = "Open in Warp")]
        warp: bool,
        #[arg(long, help = "Open in both Warp and Cursor")]
        all: bool,
    },

    #[command(about = "Print worktree path for a branch")]
    Path { branch: String },

    #[command(about = "Clean up merged/stale worktrees")]
    Clean {
        #[arg(long, help = "List candidates without removing")]
        dry_run: bool,
    },

    #[command(about = "Generate shell completions")]
    Completions { shell: ShellType },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Elvish,
    #[value(name = "powershell")]
    PowerShell,
}

impl From<ShellType> for clap_complete::Shell {
    fn from(s: ShellType) -> Self {
        match s {
            ShellType::Bash => clap_complete::Shell::Bash,
            ShellType::Zsh => clap_complete::Shell::Zsh,
            ShellType::Fish => clap_complete::Shell::Fish,
            ShellType::Elvish => clap_complete::Shell::Elvish,
            ShellType::PowerShell => clap_complete::Shell::PowerShell,
        }
    }
}
