/// Integration tests for UheGit.
///
/// These tests use real temporary Git repositories and do NOT depend on
/// network access, GitHub accounts, or the developer's personal projects.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn git(args: &[&str], dir: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .status()
        .expect("git command failed");
    assert!(
        status.success(),
        "git {:?} failed in {}",
        args,
        dir.display()
    );
}

fn git_output(args: &[&str], dir: &Path) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .output()
        .expect("git command failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_repo(path: &Path) {
    git(&["init"], path);
    git(&["config", "user.email", "test@test.com"], path);
    git(&["config", "user.name", "Test User"], path);
}

fn commit(dir: &Path, msg: &str) {
    fs::write(dir.join(".gitkeep"), "").unwrap();
    git(&["add", "."], dir);
    git(&["commit", "--allow-empty", "-m", msg], dir);
}

fn make_repo_with_commit(name: &str, root: &Path) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    init_repo(&path);
    commit(&path, "Initial commit");
    path
}

// ─── Scanner unit-level integration ─────────────────────────────────────────

mod scanner_tests {
    use super::*;
    use std::collections::HashSet;
    use uhegit::discovery::scanner::{scan_for_repos, ScanResult};

    #[test]
    fn test_scan_discovers_repos() {
        let root = TempDir::new().unwrap();
        make_repo_with_commit("alpha", root.path());
        make_repo_with_commit("beta", root.path());

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 3, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 2);
    }

    #[test]
    fn test_scan_does_not_duplicate() {
        let root = TempDir::new().unwrap();
        make_repo_with_commit("repo1", root.path());

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 3, 15, &mut result, &HashSet::new());
        scan_for_repos(root.path(), 3, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_skips_node_modules() {
        let root = TempDir::new().unwrap();
        let nm_repo = root.path().join("node_modules").join("evil-repo");
        fs::create_dir_all(&nm_repo).unwrap();
        init_repo(&nm_repo);

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 5, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 0);
    }

    #[test]
    fn test_scan_skips_target_dir() {
        let root = TempDir::new().unwrap();
        let target_repo = root.path().join("target").join("generated-repo");
        fs::create_dir_all(&target_repo).unwrap();
        init_repo(&target_repo);

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 5, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 0);
    }

    #[test]
    fn test_scan_depth_limit() {
        let root = TempDir::new().unwrap();
        let deep = root
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e");
        fs::create_dir_all(&deep).unwrap();
        init_repo(&deep);

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 2, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 0);

        let mut result2 = ScanResult::default();
        scan_for_repos(root.path(), 6, 15, &mut result2, &HashSet::new());
        assert_eq!(result2.found.len(), 1);
    }

    #[test]
    fn test_scan_path_with_spaces() {
        let root = TempDir::new().unwrap();
        let dir = root.path().join("my project with spaces");
        fs::create_dir_all(&dir).unwrap();
        init_repo(&dir);
        commit(&dir, "Init");

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 3, 15, &mut result, &HashSet::new());
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_does_not_descend_into_git_dir() {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("myrepo");
        fs::create_dir_all(&repo).unwrap();
        init_repo(&repo);

        let mut result = ScanResult::default();
        scan_for_repos(root.path(), 10, 15, &mut result, &HashSet::new());
        // Should only find myrepo, not something inside .git
        assert_eq!(result.found.len(), 1);
    }
}

// ─── Git status tests ────────────────────────────────────────────────────────

mod git_status_tests {
    use super::*;
    use uhegit::git::status::collect_status;
    use uhegit::types::StatusCategory;

    #[test]
    fn test_status_clean_repo() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("clean", root.path());
        let status = collect_status(&path, 15).unwrap();
        assert_eq!(status.staged, 0);
        assert_eq!(status.modified, 0);
        assert_eq!(status.untracked, 0);
        assert_eq!(status.category, StatusCategory::Clean);
    }

    #[test]
    fn test_status_untracked_file() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("untracked", root.path());
        fs::write(path.join("newfile.txt"), "hello").unwrap();
        let status = collect_status(&path, 15).unwrap();
        assert_eq!(status.untracked, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_status_staged_file() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("staged", root.path());
        fs::write(path.join("newfile.txt"), "hello").unwrap();
        git(&["add", "newfile.txt"], &path);
        let status = collect_status(&path, 15).unwrap();
        assert_eq!(status.staged, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_status_modified_file() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("modified", root.path());
        let f = path.join(".gitkeep");
        fs::write(&f, "modified content").unwrap();
        let status = collect_status(&path, 15).unwrap();
        assert_eq!(status.modified, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_status_deleted_file() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("deleted", root.path());
        let f = path.join(".gitkeep");
        fs::remove_file(&f).unwrap();
        let status = collect_status(&path, 15).unwrap();
        assert!(status.deleted > 0 || status.modified > 0); // deleted shows as modified in some git versions
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_status_commit_info() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("commitinfo", root.path());
        let status = collect_status(&path, 15).unwrap();
        assert!(status.commit_hash.is_some());
        assert!(status.commit_subject.is_some());
        assert!(status.commit_time_relative.is_some());
    }

    #[test]
    fn test_status_detached_head() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("detached", root.path());
        let hash = git_output(&["rev-parse", "HEAD"], &path);
        git(&["checkout", &hash], &path);
        let status = collect_status(&path, 15).unwrap();
        assert!(status.detached_head);
        assert!(status.branch.is_none());
    }

    #[test]
    fn test_status_ahead_of_upstream() {
        let root = TempDir::new().unwrap();
        // Create a bare remote
        let remote_dir = root.path().join("remote.git");
        fs::create_dir_all(&remote_dir).unwrap();
        git(&["init", "--bare"], &remote_dir);

        let local = root.path().join("local");
        fs::create_dir_all(&local).unwrap();
        init_repo(&local);
        commit(&local, "Init");
        git(
            &["remote", "add", "origin", remote_dir.to_str().unwrap()],
            &local,
        );
        git(&["push", "-u", "origin", "HEAD:main"], &local);

        // Make a new commit that hasn't been pushed
        fs::write(local.join("extra.txt"), "extra").unwrap();
        git(&["add", "."], &local);
        git(&["commit", "-m", "Extra commit"], &local);

        let status = collect_status(&local, 15).unwrap();
        assert_eq!(status.ahead, Some(1));
        assert_eq!(status.category, StatusCategory::Unpushed);
    }

    #[test]
    fn test_status_behind_upstream() {
        let root = TempDir::new().unwrap();
        let remote_dir = root.path().join("remote.git");
        fs::create_dir_all(&remote_dir).unwrap();
        git(&["init", "--bare"], &remote_dir);

        // Clone and set up
        let local = root.path().join("local");
        fs::create_dir_all(&local).unwrap();
        init_repo(&local);
        commit(&local, "Init");
        git(
            &["remote", "add", "origin", remote_dir.to_str().unwrap()],
            &local,
        );
        git(&["push", "-u", "origin", "HEAD:main"], &local);

        // Create another clone and push to remote
        let local2 = root.path().join("local2");
        fs::create_dir_all(&local2).unwrap();
        init_repo(&local2);
        commit(&local2, "Init");
        git(
            &["remote", "add", "origin", remote_dir.to_str().unwrap()],
            &local2,
        );
        git(&["fetch", "origin"], &local2);
        git(&["checkout", "-b", "main", "origin/main"], &local2);
        fs::write(local2.join("remote-change.txt"), "remote").unwrap();
        git(&["add", "."], &local2);
        git(&["commit", "-m", "Remote commit"], &local2);
        git(&["push", "origin", "main"], &local2);

        // local1 fetches but doesn't pull (so tracking ref is updated)
        git(&["fetch", "origin"], &local);

        let status = collect_status(&local, 15).unwrap();
        assert!(status.behind.is_some() && status.behind.unwrap() > 0);
        assert_eq!(status.category, StatusCategory::Behind);
    }

    #[test]
    fn test_status_does_not_modify_repo() {
        let root = TempDir::new().unwrap();
        let path = make_repo_with_commit("no-modify", root.path());
        fs::write(path.join("unstaged.txt"), "unstaged").unwrap();

        let before = git_output(&["status", "--porcelain"], &path);
        let _ = collect_status(&path, 15);
        let after = git_output(&["status", "--porcelain"], &path);

        assert_eq!(before, after, "collect_status modified the repository!");
    }

    #[test]
    fn test_status_no_commits() {
        let root = TempDir::new().unwrap();
        let path = root.path().join("empty-repo");
        fs::create_dir_all(&path).unwrap();
        init_repo(&path);
        // Don't make any commits — should not panic
        let _result = collect_status(&path, 15);
        // It's acceptable for this to return an error or a status with None commit info
    }
}

// ─── Storage tests ───────────────────────────────────────────────────────────

mod storage_tests {
    use super::*;
    use uhegit::storage::state::*;

    #[test]
    fn test_add_and_persist_repo() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("my-repo");
        fs::create_dir_all(&path).unwrap();
        assert!(add_repo(&mut state, &path, false));
        assert_eq!(state.repos.len(), 1);
    }

    #[test]
    fn test_note_branch_isolation() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        add_repo(&mut state, &path, false);

        // Must find by name
        let name = "repo";
        save_note(&mut state, name, "main", "Main note").unwrap();
        save_note(&mut state, name, "feature/x", "Feature note").unwrap();

        let repo = get_repo(&state, name).unwrap();
        assert_eq!(repo.sessions["main"].note.as_deref(), Some("Main note"));
        assert_eq!(
            repo.sessions["feature/x"].note.as_deref(),
            Some("Feature note")
        );

        // Changing branch does not overwrite main note
        save_note(&mut state, name, "main", "Updated main").unwrap();
        let repo = get_repo(&state, name).unwrap();
        assert_eq!(
            repo.sessions["feature/x"].note.as_deref(),
            Some("Feature note")
        );
        assert_eq!(repo.sessions["main"].note.as_deref(), Some("Updated main"));
    }

    #[test]
    fn test_checkpoint_persists() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        add_repo(&mut state, &path, false);

        save_checkpoint(&mut state, "repo", "main", "abc1234", "2 modified").unwrap();
        let repo = get_repo(&state, "repo").unwrap();
        assert_eq!(
            repo.sessions["main"].checkpoint_hash.as_deref(),
            Some("abc1234")
        );
        assert_eq!(
            repo.sessions["main"].checkpoint_summary.as_deref(),
            Some("2 modified")
        );
    }

    #[test]
    fn test_forget_clears_sessions() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        add_repo(&mut state, &path, false);
        save_note(&mut state, "repo", "main", "Some note").unwrap();
        forget_repo_sessions(&mut state, "repo").unwrap();
        let repo = get_repo(&state, "repo").unwrap();
        assert!(repo.sessions.is_empty());
    }

    #[test]
    fn test_remove_does_not_delete_files() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        add_repo(&mut state, &path, false);
        remove_repo(&mut state, "repo").unwrap();
        assert_eq!(state.repos.len(), 0);
        // The directory should still exist
        assert!(path.exists());
    }

    #[test]
    fn test_validate_marks_inaccessible() {
        let dir = TempDir::new().unwrap();
        let mut state = AppState::default();
        let path = dir.path().join("repo");
        fs::create_dir_all(&path).unwrap();
        add_repo(&mut state, &path, false);

        // Remove the directory to simulate it being moved/deleted
        fs::remove_dir_all(&path).unwrap();
        validate_repos(&mut state);

        let repo = state.repos.values().next().unwrap();
        assert!(repo.inaccessible);
        // Session data is NOT deleted
    }
}

// ─── Parser unit tests via re-export ─────────────────────────────────────────

mod parser_tests {
    use uhegit::git::parser::parse_porcelain_v1;
    use uhegit::types::StatusCategory;

    #[test]
    fn test_parse_file_with_spaces_in_name() {
        // NUL-separated: branch header, then file with space in name
        let mut input = b"## main\0".to_vec();
        input.extend_from_slice(b"?? my file with spaces.txt\0");
        let status = parse_porcelain_v1(&input);
        assert_eq!(status.untracked, 1);
    }

    #[test]
    fn test_parse_multiple_changes() {
        let mut input = b"## main\0".to_vec();
        input.extend_from_slice(b"M  staged.rs\0");
        input.extend_from_slice(b" M modified.rs\0");
        input.extend_from_slice(b"?? untracked.txt\0");
        let status = parse_porcelain_v1(&input);
        assert_eq!(status.staged, 1);
        assert_eq!(status.modified, 1);
        assert_eq!(status.untracked, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }
}

// ─── JSON output tests ───────────────────────────────────────────────────────

mod json_tests {
    use std::path::PathBuf;
    use uhegit::output::json::{repo_to_json, repos_to_json};
    use uhegit::types::*;

    fn make_clean_repo() -> Repository {
        Repository {
            name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
            git_status: Some(GitStatus {
                branch: Some("main".to_string()),
                detached_head: false,
                commit_hash: Some("abc1234".to_string()),
                commit_subject: Some("Initial commit".to_string()),
                commit_time_relative: Some("5 minutes ago".to_string()),
                staged: 0,
                modified: 0,
                untracked: 0,
                conflicted: 0,
                deleted: 0,
                upstream: None,
                ahead: None,
                behind: None,
                category: StatusCategory::Clean,
                sync_stale: true,
            }),
            session: None,
            error: None,
        }
    }

    #[test]
    fn test_json_is_valid() {
        let repo = make_clean_repo();
        let workspace = repos_to_json(&[repo]);
        let json = serde_json::to_string_pretty(&workspace).unwrap();
        // Must parse back cleanly
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["repositories"].is_array());
        assert_eq!(parsed["summary"]["total"], 1);
        assert_eq!(parsed["summary"]["clean"], 1);
    }

    #[test]
    fn test_json_null_for_missing_git_status() {
        let repo = Repository {
            name: "broken".to_string(),
            path: PathBuf::from("/tmp/broken"),
            git_status: None,
            session: None,
            error: Some("git failed".to_string()),
        };
        let json_repo = repo_to_json(&repo);
        let json = serde_json::to_string(&json_repo).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["git"].is_null());
        assert_eq!(parsed["error"], "git failed");
    }

    #[test]
    fn test_json_multiple_repos() {
        let repos = vec![make_clean_repo(), make_clean_repo()];
        let workspace = repos_to_json(&repos);
        let json = serde_json::to_string(&workspace).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["total"], 2);
    }
}
