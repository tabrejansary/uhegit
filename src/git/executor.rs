use crate::error::UheGitError;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

/// Default timeout for Git commands in seconds.
pub const GIT_TIMEOUT_SECS: u64 = 15;

/// Execute a Git command with a timeout, capturing stdout and stderr.
/// Never interpolates user input into shell strings — uses argument arrays.
pub fn run_git(args: &[&str], working_dir: &Path, timeout_secs: u64) -> Result<Output> {
    let git = find_git()?;

    // Use a thread to implement timeout since std::process doesn't support it directly
    let mut cmd = Command::new(&git);
    cmd.args(args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Disable interactive prompts and credential helpers that could block
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "echo")
        // Prevent locale-specific output
        .env("LANG", "C")
        .env("LC_ALL", "C");

    let child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn git process for: {}", working_dir.display()))?;

    run_with_timeout(child, timeout_secs, working_dir)
}

/// Run a git command using -C <path> (without changing working directory of caller).
pub fn run_git_c(path: &Path, args: &[&str], timeout_secs: u64) -> Result<Output> {
    let git = find_git()?;
    let path_str = path.to_str().unwrap_or(".");

    let mut full_args = vec!["-C", path_str];
    full_args.extend_from_slice(args);

    let mut cmd = Command::new(&git);
    cmd.args(&full_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "echo")
        .env("LANG", "C")
        .env("LC_ALL", "C");

    let child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn git process for: {}", path.display()))?;

    run_with_timeout(child, timeout_secs, path)
}

fn run_with_timeout(child: std::process::Child, timeout_secs: u64, path: &Path) -> Result<Output> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    let child_id = child.id();

    // Spawn a thread that waits for the child to finish
    thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(result) => result.with_context(|| format!("Git command failed in: {}", path.display())),
        Err(_) => {
            // Try to kill the child process
            #[cfg(unix)]
            unsafe {
                libc::kill(child_id as i32, libc::SIGKILL);
            }
            #[cfg(windows)]
            {
                // On Windows, try to kill via taskkill
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &child_id.to_string()])
                    .output();
            }
            Err(UheGitError::GitTimeout {
                seconds: timeout_secs,
                path: path.display().to_string(),
            }
            .into())
        }
    }
}

/// Find the git executable path.
pub fn find_git() -> Result<String> {
    // Check if git is accessible by running git --version
    let result = Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => Ok("git".to_string()),
        _ => Err(UheGitError::GitNotFound.into()),
    }
}

/// Check if Git is available in the PATH.
pub fn check_git_available() -> Result<()> {
    find_git().map(|_| ())
}

/// Parse UTF-8 output from a git command, treating non-UTF8 lossily.
pub fn output_to_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Parse UTF-8 stderr from a git command.
pub fn stderr_to_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(&["init"], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        run_git(
            &["config", "user.email", "test@example.com"],
            dir.path(),
            GIT_TIMEOUT_SECS,
        )
        .unwrap();
        run_git(
            &["config", "user.name", "Test User"],
            dir.path(),
            GIT_TIMEOUT_SECS,
        )
        .unwrap();
        dir
    }

    #[test]
    fn test_git_available() {
        assert!(check_git_available().is_ok());
    }

    #[test]
    fn test_run_git_init() {
        let dir = TempDir::new().unwrap();
        let output = run_git(&["init"], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_run_git_c() {
        let dir = init_test_repo();
        let output = run_git_c(dir.path(), &["rev-parse", "--git-dir"], GIT_TIMEOUT_SECS).unwrap();
        assert!(output.status.success());
        let out = output_to_string(&output).trim().to_string();
        assert_eq!(out, ".git");
    }
}
