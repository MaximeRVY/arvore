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
