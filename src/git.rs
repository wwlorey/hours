use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config::GitConfig;

fn is_git_disabled(no_git_flag: bool) -> bool {
    if no_git_flag {
        return true;
    }
    std::env::var("HOURS_NO_GIT").ok().as_deref() == Some("1")
}

fn git_binary_exists() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn run_git(data_dir: &Path, args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new("git")
        .arg("-C")
        .arg(data_dir)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;
    Ok(output)
}

fn run_git_checked(data_dir: &Path, args: &[&str]) -> Result<()> {
    let output = run_git(data_dir, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(())
}

fn is_git_repo(data_dir: &Path) -> bool {
    run_git(data_dir, &["rev-parse", "--git-dir"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn git_init(data_dir: &Path, remote_name: &str, remote_url: &str) -> Result<()> {
    if !git_binary_exists() {
        bail!("git is not installed. Install git and try again.");
    }

    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("Failed to create data directory {}", data_dir.display()))?;

    if !is_git_repo(data_dir) {
        run_git_checked(data_dir, &["init"])?;
    }

    let remote_check = run_git(data_dir, &["remote", "get-url", remote_name])?;
    if !remote_check.status.success() {
        run_git_checked(data_dir, &["remote", "add", remote_name, remote_url])?;
    }

    let gitignore_path = data_dir.join(".gitignore");
    std::fs::write(&gitignore_path, "*.tmp\nexports/\n").context("Failed to write .gitignore")?;

    Ok(())
}

pub fn git_commit(data_dir: &Path, message: &str) -> Result<()> {
    if !is_git_repo(data_dir) {
        bail!("Data directory is not a git repository. Run 'hours init' to set up.");
    }

    run_git_checked(data_dir, &["add", "hours.json"])?;

    if data_dir.join(".gitignore").exists() {
        let _ = run_git(data_dir, &["add", ".gitignore"]);
    }

    let output = run_git(data_dir, &["commit", "-m", message])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stderr.contains("nothing to commit") || stdout.contains("nothing to commit") {
            return Ok(());
        }
        bail!("git commit failed: {}", stderr.trim());
    }
    Ok(())
}

fn current_branch(data_dir: &Path) -> Result<String> {
    let output = run_git(data_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if !output.status.success() {
        bail!("Failed to determine current branch");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn git_push(data_dir: &Path, remote: &str) -> Result<()> {
    let branch = current_branch(data_dir).unwrap_or_else(|_| "main".to_string());
    let output = run_git(data_dir, &["push", "-u", remote, &branch])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "Warning: git push failed: {}. Data saved locally.",
            stderr.trim()
        );
    }
    Ok(())
}

pub fn git_sync(data_dir: &Path, config: &GitConfig, message: &str, no_git: bool) -> Result<()> {
    if is_git_disabled(no_git) {
        return Ok(());
    }

    if !git_binary_exists() {
        eprintln!("Warning: git is not installed. Data is saved locally only.");
        return Ok(());
    }

    if !is_git_repo(data_dir) {
        bail!("Data directory is not a git repository. Run 'hours init' to set up.");
    }

    git_commit(data_dir, message)?;

    if config.auto_push {
        let remote_check = run_git(data_dir, &["remote"])?;
        let remotes = String::from_utf8_lossy(&remote_check.stdout);
        if remotes.trim().is_empty() {
            eprintln!("Warning: No git remote configured. Data is saved locally only.");
        } else {
            git_push(data_dir, &config.remote)?;
        }
    }

    Ok(())
}

pub fn git_init_and_commit(
    data_dir: &Path,
    config: &GitConfig,
    remote_url: &str,
    no_git: bool,
) -> Result<()> {
    if is_git_disabled(no_git) {
        return Ok(());
    }

    git_init(data_dir, &config.remote, remote_url)?;
    git_commit(data_dir, "Initialize hours tracking")?;

    if config.auto_push {
        git_push(data_dir, &config.remote)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn set_git_test_config(dir: &Path) {
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn setup_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        set_git_test_config(dir);
    }

    #[test]
    fn is_git_disabled_flag_true() {
        assert!(is_git_disabled(true));
    }

    #[test]
    fn is_git_disabled_flag_false_no_env() {
        let prev = std::env::var("HOURS_NO_GIT").ok();
        std::env::remove_var("HOURS_NO_GIT");
        assert!(!is_git_disabled(false));
        if let Some(val) = prev {
            std::env::set_var("HOURS_NO_GIT", val);
        }
    }

    #[test]
    fn git_binary_exists_returns_true() {
        assert!(git_binary_exists());
    }

    #[test]
    fn is_git_repo_false_for_plain_dir() {
        let tmp = TempDir::new().unwrap();
        assert!(!is_git_repo(tmp.path()));
    }

    #[test]
    fn is_git_repo_true_after_init() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo(tmp.path());
        assert!(is_git_repo(tmp.path()));
    }

    #[test]
    fn git_init_creates_repo_and_gitignore() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("data");
        git_init(&data_dir, "origin", "git@example.com:test/test.git").unwrap();
        assert!(is_git_repo(&data_dir));
        assert!(data_dir.join(".gitignore").exists());
        let gitignore = std::fs::read_to_string(data_dir.join(".gitignore")).unwrap();
        assert!(gitignore.contains("*.tmp"));
        assert!(gitignore.contains("exports/"));
    }

    #[test]
    fn git_init_idempotent() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("data");
        git_init(&data_dir, "origin", "git@example.com:test/test.git").unwrap();
        git_init(&data_dir, "origin", "git@example.com:test/test.git").unwrap();
        assert!(is_git_repo(&data_dir));
    }

    #[test]
    fn git_commit_with_data_file() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();
        git_commit(data_dir, "Test commit").unwrap();

        let log = run_git(data_dir, &["log", "--oneline"]).unwrap();
        let log_text = String::from_utf8_lossy(&log.stdout);
        assert!(log_text.contains("Test commit"));
    }

    #[test]
    fn git_commit_nothing_to_commit_is_ok() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();
        git_commit(data_dir, "First commit").unwrap();
        let result = git_commit(data_dir, "Nothing changed");
        assert!(result.is_ok());
    }

    #[test]
    fn git_commit_fails_if_not_repo() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("hours.json"), r#"{"weeks":[]}"#).unwrap();
        let result = git_commit(tmp.path(), "Should fail");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a git repository"));
    }

    #[test]
    fn git_sync_noop_when_disabled_by_flag() {
        let tmp = TempDir::new().unwrap();
        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: true,
        };
        let result = git_sync(tmp.path(), &config, "test", true);
        assert!(result.is_ok());
    }

    #[test]
    fn git_sync_commits_file() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();

        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: false,
        };
        git_sync(data_dir, &config, "Sync commit", false).unwrap();

        let log = run_git(data_dir, &["log", "--oneline"]).unwrap();
        let log_text = String::from_utf8_lossy(&log.stdout);
        assert!(log_text.contains("Sync commit"));
    }

    #[test]
    fn git_sync_no_push_when_auto_push_disabled() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();

        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: false,
        };
        git_sync(data_dir, &config, "No push", false).unwrap();

        let log = run_git(data_dir, &["log", "--oneline"]).unwrap();
        let log_text = String::from_utf8_lossy(&log.stdout);
        assert!(log_text.contains("No push"));
    }

    #[test]
    fn git_init_and_commit_full_flow() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("data");

        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();

        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: false,
        };

        git_init(&data_dir, &config.remote, "git@example.com:test/test.git").unwrap();
        set_git_test_config(&data_dir);

        git_commit(&data_dir, "Initialize hours tracking").unwrap();

        assert!(is_git_repo(&data_dir));
        let log = run_git(&data_dir, &["log", "--oneline"]).unwrap();
        let log_text = String::from_utf8_lossy(&log.stdout);
        assert!(log_text.contains("Initialize hours tracking"));
    }

    #[test]
    fn git_init_and_commit_noop_when_disabled() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("data");

        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: true,
        };
        let result = git_init_and_commit(&data_dir, &config, "git@example.com:test/test.git", true);
        assert!(result.is_ok());
        assert!(!data_dir.exists());
    }

    #[test]
    fn git_push_warns_on_failure() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();
        git_commit(data_dir, "test").unwrap();

        let result = git_push(data_dir, "origin");
        assert!(result.is_ok());
    }

    #[test]
    fn git_sync_warns_no_remote() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path();
        setup_git_repo(data_dir);

        std::fs::write(data_dir.join("hours.json"), r#"{"weeks":[]}"#).unwrap();

        let config = GitConfig {
            remote: "origin".to_string(),
            auto_push: true,
        };
        let result = git_sync(data_dir, &config, "test", false);
        assert!(result.is_ok());
    }
}
