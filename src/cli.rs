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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_create_basic() {
        let cli = Cli::try_parse_from(["arvore", "create", "my-branch"]).unwrap();
        match cli.command {
            Commands::Create { branch, from, open } => {
                assert_eq!(branch, "my-branch");
                assert!(from.is_none());
                assert!(!open);
            }
            _ => panic!("expected Create"),
        }
    }

    #[test]
    fn parse_create_with_from() {
        let cli = Cli::try_parse_from(["arvore", "create", "my-branch", "--from", "main"]).unwrap();
        match cli.command {
            Commands::Create { from, .. } => assert_eq!(from.as_deref(), Some("main")),
            _ => panic!("expected Create"),
        }
    }

    #[test]
    fn parse_create_with_open() {
        let cli = Cli::try_parse_from(["arvore", "create", "my-branch", "--open"]).unwrap();
        match cli.command {
            Commands::Create { open, .. } => assert!(open),
            _ => panic!("expected Create"),
        }
    }

    #[test]
    fn parse_list() {
        let cli = Cli::try_parse_from(["arvore", "ls"]).unwrap();
        match cli.command {
            Commands::List { porcelain } => assert!(!porcelain),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_list_porcelain() {
        let cli = Cli::try_parse_from(["arvore", "ls", "--porcelain"]).unwrap();
        match cli.command {
            Commands::List { porcelain } => assert!(porcelain),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_remove() {
        let cli = Cli::try_parse_from(["arvore", "rm", "my-branch"]).unwrap();
        match cli.command {
            Commands::Remove { target, force } => {
                assert_eq!(target, "my-branch");
                assert!(!force);
            }
            _ => panic!("expected Remove"),
        }
    }

    #[test]
    fn parse_remove_force() {
        let cli = Cli::try_parse_from(["arvore", "rm", "my-branch", "--force"]).unwrap();
        match cli.command {
            Commands::Remove { force, .. } => assert!(force),
            _ => panic!("expected Remove"),
        }
    }

    #[test]
    fn parse_open_cursor() {
        let cli = Cli::try_parse_from(["arvore", "open", "my-branch", "--cursor"]).unwrap();
        match cli.command {
            Commands::Open {
                branch,
                cursor,
                all,
                ..
            } => {
                assert_eq!(branch, "my-branch");
                assert!(cursor);
                assert!(!all);
            }
            _ => panic!("expected Open"),
        }
    }

    #[test]
    fn parse_open_all() {
        let cli = Cli::try_parse_from(["arvore", "open", "my-branch", "--all"]).unwrap();
        match cli.command {
            Commands::Open { all, .. } => assert!(all),
            _ => panic!("expected Open"),
        }
    }

    #[test]
    fn parse_path() {
        let cli = Cli::try_parse_from(["arvore", "path", "my-branch"]).unwrap();
        match cli.command {
            Commands::Path { branch } => assert_eq!(branch, "my-branch"),
            _ => panic!("expected Path"),
        }
    }

    #[test]
    fn parse_clean() {
        let cli = Cli::try_parse_from(["arvore", "clean"]).unwrap();
        match cli.command {
            Commands::Clean { dry_run } => assert!(!dry_run),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn parse_clean_dry_run() {
        let cli = Cli::try_parse_from(["arvore", "clean", "--dry-run"]).unwrap();
        match cli.command {
            Commands::Clean { dry_run } => assert!(dry_run),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn parse_completions_zsh() {
        let cli = Cli::try_parse_from(["arvore", "completions", "zsh"]).unwrap();
        match cli.command {
            Commands::Completions { shell } => {
                let s: clap_complete::Shell = shell.into();
                assert_eq!(s, clap_complete::Shell::Zsh);
            }
            _ => panic!("expected Completions"),
        }
    }

    #[test]
    fn parse_global_config_flag() {
        let cli = Cli::try_parse_from(["arvore", "--config", "/custom/path", "ls"]).unwrap();
        assert_eq!(cli.config, Some(std::path::PathBuf::from("/custom/path")));
    }

    #[test]
    fn parse_missing_subcommand_errors() {
        let result = Cli::try_parse_from(["arvore"]);
        assert!(result.is_err());
    }
}
