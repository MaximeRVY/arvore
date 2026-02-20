use std::path::{Path, PathBuf};
use std::process::Command;

fn arvore_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_arvore"))
}

fn setup_test_repo() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(&path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(&path)
        .output()
        .unwrap();

    (dir, path)
}

fn write_test_config(dir: &Path, worktree_base: &Path) -> PathBuf {
    let config_path = dir.join("config.yaml");
    let canonical_base = worktree_base
        .canonicalize()
        .unwrap_or_else(|_| worktree_base.to_path_buf());
    std::fs::write(
        &config_path,
        format!("worktree_base: {}", canonical_base.display()),
    )
    .unwrap();
    config_path
}

fn run_arvore(repo: &Path, config: &Path, args: &[&str]) -> std::process::Output {
    let mut cmd_args = vec!["--config", config.to_str().unwrap()];
    cmd_args.extend_from_slice(args);

    Command::new(arvore_bin())
        .args(&cmd_args)
        .current_dir(repo)
        .output()
        .unwrap()
}

#[test]
fn ls_shows_main_worktree() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(&repo, &config, &["ls", "--porcelain"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("main") || stdout.contains("master"),
        "stdout: {stdout}"
    );
}

#[test]
fn ls_porcelain_tab_separated() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(&repo, &config, &["ls", "--porcelain"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            4,
            "expected 4 tab-separated fields, got: {line}"
        );
    }
}

#[test]
fn path_outputs_expected_path() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(&repo, &config, &["path", "feature-x"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_name = repo.file_name().unwrap().to_string_lossy();
    let canonical_base = wt_base.path().canonicalize().unwrap();
    let expected = canonical_base.join(repo_name.as_ref()).join("feature-x");
    assert_eq!(stdout, expected.to_string_lossy());
}

#[test]
fn create_ls_rm_lifecycle() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(&repo, &config, &["create", "test-branch", "--from", "main"]);
    assert!(
        output.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = run_arvore(&repo, &config, &["ls", "--porcelain"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test-branch"),
        "branch not in ls output: {stdout}"
    );

    let output = run_arvore(&repo, &config, &["rm", "test-branch"]);
    assert!(
        output.status.success(),
        "rm failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = run_arvore(&repo, &config, &["ls", "--porcelain"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("test-branch"),
        "branch still in ls output after rm: {stdout}"
    );
}

#[test]
fn rm_dirty_worktree_without_force_fails() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(
        &repo,
        &config,
        &["create", "dirty-branch", "--from", "main"],
    );
    assert!(
        output.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_name = repo.file_name().unwrap().to_string_lossy();
    let wt_path = wt_base.path().join(repo_name.as_ref()).join("dirty-branch");
    std::fs::write(wt_path.join("dirty.txt"), "uncommitted").unwrap();

    Command::new("git")
        .args(["add", "dirty.txt"])
        .current_dir(&wt_path)
        .output()
        .unwrap();

    let output = run_arvore(&repo, &config, &["rm", "dirty-branch"]);
    assert!(!output.status.success(), "rm should fail on dirty worktree");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--force"),
        "stderr should mention --force: {stderr}"
    );
}

#[test]
fn completions_zsh_produces_output() {
    let (_dir, repo) = setup_test_repo();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(_dir.path(), wt_base.path());

    let output = run_arvore(&repo, &config, &["completions", "zsh"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.starts_with("#compdef"),
        "zsh completions should start with #compdef, got: {}",
        &stdout[..stdout.len().min(50)]
    );
}

#[test]
fn ls_outside_git_repo_fails() {
    let dir = tempfile::tempdir().unwrap();
    let wt_base = tempfile::tempdir().unwrap();
    let config = write_test_config(dir.path(), wt_base.path());

    let output = run_arvore(dir.path(), &config, &["ls"]);
    assert!(!output.status.success(), "ls should fail outside git repo");
}
