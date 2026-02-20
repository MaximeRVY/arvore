use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::error::ArvoreError;

#[derive(Debug, Clone)]
pub struct Config {
    pub worktree_base: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    worktree_base: Option<String>,
}

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let path = match config_path {
            Some(p) => p.to_path_buf(),
            None => default_config_path()?,
        };

        if path.exists() {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file: {}", path.display()))?;
            let raw: RawConfig = serde_yml::from_str(&contents)
                .map_err(|e| ArvoreError::ConfigError(e.to_string()))?;
            let base = raw
                .worktree_base
                .unwrap_or_else(|| "~/Dev/worktrees".to_string());
            Ok(Config {
                worktree_base: expand_tilde(&base),
            })
        } else {
            Ok(Config::default())
        }
    }

    pub fn worktree_path(&self, repo_name: &str, branch: &str) -> PathBuf {
        let sanitized = branch.replace('/', "-");
        self.worktree_base.join(repo_name).join(sanitized)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            worktree_base: expand_tilde("~/Dev/worktrees"),
        }
    }
}

fn default_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| ArvoreError::ConfigError("cannot determine home directory".into()))?;
    Ok(home.join(".config").join("arvore").join("config.yaml"))
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(stripped);
    }
    if path == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_with_subpath() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~/foo"), home.join("foo"));
    }

    #[test]
    fn expand_tilde_alone() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~"), home);
    }

    #[test]
    fn expand_tilde_absolute_unchanged() {
        assert_eq!(expand_tilde("/absolute/path"), PathBuf::from("/absolute/path"));
    }

    #[test]
    fn expand_tilde_relative_unchanged() {
        assert_eq!(expand_tilde("relative/path"), PathBuf::from("relative/path"));
    }

    #[test]
    fn worktree_path_simple_branch() {
        let config = Config {
            worktree_base: PathBuf::from("/base"),
        };
        assert_eq!(
            config.worktree_path("myrepo", "feature"),
            PathBuf::from("/base/myrepo/feature")
        );
    }

    #[test]
    fn worktree_path_slash_in_branch() {
        let config = Config {
            worktree_base: PathBuf::from("/base"),
        };
        assert_eq!(
            config.worktree_path("myrepo", "feature/auth"),
            PathBuf::from("/base/myrepo/feature-auth")
        );
    }

    #[test]
    fn worktree_path_multiple_slashes() {
        let config = Config {
            worktree_base: PathBuf::from("/base"),
        };
        assert_eq!(
            config.worktree_path("myrepo", "feat/sub/deep"),
            PathBuf::from("/base/myrepo/feat-sub-deep")
        );
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("nonexistent.yaml");
        let config = Config::load(Some(&config_path)).unwrap();
        let default = Config::default();
        assert_eq!(config.worktree_base, default.worktree_base);
    }

    #[test]
    fn load_valid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "worktree_base: /tmp/my-worktrees\n").unwrap();
        let config = Config::load(Some(&config_path)).unwrap();
        assert_eq!(config.worktree_base, PathBuf::from("/tmp/my-worktrees"));
    }

    #[test]
    fn load_yaml_without_worktree_base_uses_default() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "other_key: value\n").unwrap();
        let config = Config::load(Some(&config_path)).unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(config.worktree_base, home.join("Dev/worktrees"));
    }

    #[test]
    fn load_invalid_yaml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "worktree_base: [invalid\n").unwrap();
        let result = Config::load(Some(&config_path));
        assert!(result.is_err());
    }
}
