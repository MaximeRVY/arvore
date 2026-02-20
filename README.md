# arvore

**arvore** (`/ˈaʁ.vo.ɾi/`) - from Portuguese *árvore*, meaning **tree**.

A fast CLI for managing git worktrees.

## Install

```bash
cargo install --path .
```

Or with Homebrew (coming soon):

```bash
brew install MaximeRVY/tap/arvore
```

## Usage

```
arvore create <branch> [--from <ref>] [--open]   Create a new worktree
arvore ls [--porcelain]                           List all worktrees
arvore rm <branch|path> [--force]                 Remove a worktree
arvore open <branch> [--cursor] [--warp] [--all]  Open in editor/terminal
arvore path <branch>                              Print worktree path
arvore clean [--dry-run]                          Clean up merged/stale worktrees
arvore completions <shell>                        Generate shell completions
```

### Create a worktree

```bash
# New branch from HEAD
arvore create feature-auth

# New branch from a specific ref
arvore create feature-auth --from main

# Create and open in Warp + Cursor
arvore create feature-auth --open
```

### List worktrees

```bash
arvore ls
```

```
  fd8fc24e main     ~/Dev/worktrees/myapp/main
  a1b2c3d4 feature  ~/Dev/worktrees/myapp/feature [modified]
```

### Remove a worktree

```bash
arvore rm feature-auth

# Force remove if there are uncommitted changes
arvore rm feature-auth --force
```

### Clean up stale worktrees

```bash
# Preview what would be cleaned
arvore clean --dry-run

# Interactive cleanup of merged/stale branches
arvore clean
```

### Shell integration

Add to your `~/.zshrc`:

```zsh
wtcd() { cd "$(arvore path "$1")" }
```

Then: `wtcd feature-auth`

### Shell completions

```bash
# Zsh
arvore completions zsh > ~/.zfunc/_arvore

# Bash
arvore completions bash > /etc/bash_completion.d/arvore

# Fish
arvore completions fish > ~/.config/fish/completions/arvore.fish
```

## Configuration

Config file: `~/.config/arvore/config.yaml`

```yaml
worktree_base: ~/Dev/worktrees
```

All worktrees are created under `{worktree_base}/{repo_name}/{branch_name}`.

Branch names with `/` are sanitized to `-` (e.g. `feature/auth` becomes `feature-auth`).

## License

MIT
