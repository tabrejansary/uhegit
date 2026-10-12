use crate::error::UheGitError;
use crate::git::executor::{output_to_string, run_git_c};
use crate::git::parser::parse_porcelain_v1;
use crate::types::GitStatus;
use anyhow::Result;
use std::path::Path;

/// Collect full GitStatus for a repository at the given path.
/// Returns an error if the path is not a Git repository or Git fails.
/// Individual sub-command failures degrade gracefully (None fields).
pub fn collect_status(path: &Path, timeout_secs: u64) -> Result<GitStatus> {
    // Verify this is a valid git repo first
    let toplevel_out = run_git_c(path, &["rev-parse", "--show-toplevel"], timeout_secs)?;
    if !toplevel_out.status.success() {
        return Err(UheGitError::NotARepository {
            path: path.display().to_string(),
        }
        .into());
    }

    // Run porcelain status with branch info and NUL separators
    let status_out = run_git_c(
        path,
        &["status", "--porcelain=v1", "-z", "--branch"],
        timeout_secs,
    )?;

    let mut git_status = parse_porcelain_v1(&status_out.stdout);

    // Collect commit hash and subject
    let log_out = run_git_c(
        path,
        &["log", "-1", "--format=%h%n%s%n%cr", "--"],
        timeout_secs,
    );
    if let Ok(out) = log_out {
        if out.status.success() {
            let text = output_to_string(&out);
            let mut lines = text.lines();
            git_status.commit_hash = lines
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            git_status.commit_subject = lines
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            git_status.commit_time_relative = lines
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
    }

    Ok(git_status)
}

/// Get the canonical working-tree root for a path.
pub fn get_toplevel(path: &Path, timeout_secs: u64) -> Result<std::path::PathBuf> {
    let out = run_git_c(path, &["rev-parse", "--show-toplevel"], timeout_secs)?;
    if !out.status.success() {
        return Err(UheGitError::NotARepository {
            path: path.display().to_string(),
        }
        .into());
    }
    let s = output_to_string(&out);
    let s = s.trim();
    Ok(std::path::PathBuf::from(s))
}

/// Check if a path is inside a Git working tree.
pub fn is_git_repo(path: &Path, timeout_secs: u64) -> bool {
    let Ok(out) = run_git_c(path, &["rev-parse", "--git-dir"], timeout_secs) else {
        return false;
    };
    out.status.success()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::executor::{run_git, GIT_TIMEOUT_SECS};
    use std::fs;
    use tempfile::TempDir;

    fn make_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(&["init"], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        run_git(
            &["config", "user.email", "test@example.com"],
            dir.path(),
            GIT_TIMEOUT_SECS,
        )
        .unwrap();
        run_git(
            &["config", "user.name", "Test"],
            dir.path(),
            GIT_TIMEOUT_SECS,
        )
        .unwrap();
        dir
    }

    fn make_initial_commit(dir: &Path) {
        fs::write(dir.join("README.md"), "# Test").unwrap();
        run_git(&["add", "."], dir, GIT_TIMEOUT_SECS).unwrap();
        run_git(&["commit", "-m", "Initial commit"], dir, GIT_TIMEOUT_SECS).unwrap();
    }

    #[test]
    fn test_is_git_repo_true() {
        let dir = make_test_repo();
        assert!(is_git_repo(dir.path(), GIT_TIMEOUT_SECS));
    }

    #[test]
    fn test_is_git_repo_false() {
        let dir = TempDir::new().unwrap();
        assert!(!is_git_repo(dir.path(), GIT_TIMEOUT_SECS));
    }

    #[test]
    fn test_get_toplevel() {
        let dir = make_test_repo();
        let toplevel = get_toplevel(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        // Normalize both paths to handle any case differences or symlinks
        let expected = dunce_canonicalize(dir.path());
        let actual = dunce_canonicalize(&toplevel);
        assert_eq!(actual, expected);
    }

    fn dunce_canonicalize(p: &Path) -> std::path::PathBuf {
        std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
    }

    #[test]
    fn test_collect_status_clean() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        let status = collect_status(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert_eq!(status.staged, 0);
        assert_eq!(status.modified, 0);
        assert_eq!(status.untracked, 0);
        assert!(status.commit_hash.is_some());
    }

    #[test]
    fn test_collect_status_untracked() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        fs::write(dir.path().join("new.txt"), "hello").unwrap();
        let status = collect_status(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert_eq!(status.untracked, 1);
    }

    #[test]
    fn test_collect_status_staged() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        fs::write(dir.path().join("new.txt"), "hello").unwrap();
        run_git(&["add", "new.txt"], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        let status = collect_status(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert_eq!(status.staged, 1);
    }

    #[test]
    fn test_collect_status_modified() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        fs::write(dir.path().join("README.md"), "changed").unwrap();
        let status = collect_status(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert_eq!(status.modified, 1);
    }

    #[test]
    fn test_collect_status_detached_head() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        // Get the commit hash
        let out = run_git(&["rev-parse", "HEAD"], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
        run_git(&["checkout", &hash], dir.path(), GIT_TIMEOUT_SECS).unwrap();
        let status = collect_status(dir.path(), GIT_TIMEOUT_SECS).unwrap();
        assert!(status.detached_head);
    }

    #[test]
    fn test_status_does_not_modify_repo() {
        let dir = make_test_repo();
        make_initial_commit(dir.path());
        fs::write(dir.path().join("unstaged.txt"), "unstaged").unwrap();

        // Record state before
        let before = run_git(&["status", "--porcelain"], dir.path(), GIT_TIMEOUT_SECS).unwrap();

        // Collect status
        let _ = collect_status(dir.path(), GIT_TIMEOUT_SECS);

        // Record state after
        let after = run_git(&["status", "--porcelain"], dir.path(), GIT_TIMEOUT_SECS).unwrap();

        assert_eq!(
            before.stdout, after.stdout,
            "collect_status modified the repo!"
        );
    }
}
