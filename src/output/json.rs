use crate::types::{GitStatus, Repository, SessionMemory};
use serde::{Deserialize, Serialize};

/// JSON output structures — stable field names.

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonWorkspace {
    pub repositories: Vec<JsonRepo>,
    pub summary: JsonSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRepo {
    pub name: String,
    pub path: String,
    pub git: Option<JsonGitStatus>,
    pub session: Option<JsonSession>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonGitStatus {
    pub branch: Option<String>,
    pub detached_head: bool,
    pub commit_hash: Option<String>,
    pub commit_subject: Option<String>,
    pub commit_time_relative: Option<String>,
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
    pub conflicted: u32,
    pub deleted: u32,
    pub upstream: Option<String>,
    pub ahead: Option<u32>,
    pub behind: Option<u32>,
    pub status: String,
    pub sync_stale: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonSession {
    pub branch: Option<String>,
    pub last_session: Option<String>,
    pub note: Option<String>,
    pub next_task: Option<String>,
    pub checkpoint_hash: Option<String>,
    pub checkpoint_time: Option<String>,
    pub checkpoint_summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonSummary {
    pub total: usize,
    pub clean: usize,
    pub dirty: usize,
    pub needs_attention: usize,
    pub with_errors: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResumeOutput {
    pub name: String,
    pub path: String,
    pub git: Option<JsonGitStatus>,
    pub session: Option<JsonSession>,
}

pub fn repos_to_json(repos: &[Repository]) -> JsonWorkspace {
    let repositories: Vec<JsonRepo> = repos.iter().map(repo_to_json).collect();
    let clean = repositories
        .iter()
        .filter(|r| r.git.as_ref().map(|g| g.status == "Clean").unwrap_or(false))
        .count();
    let dirty = repositories
        .iter()
        .filter(|r| {
            r.git
                .as_ref()
                .map(|g| g.status == "Uncommitted changes")
                .unwrap_or(false)
        })
        .count();
    let with_errors = repositories.iter().filter(|r| r.error.is_some()).count();
    let total = repositories.len();

    JsonWorkspace {
        repositories,
        summary: JsonSummary {
            total,
            clean,
            dirty,
            needs_attention: total - clean,
            with_errors,
        },
    }
}

pub fn repo_to_json(repo: &Repository) -> JsonRepo {
    JsonRepo {
        name: repo.name.clone(),
        path: repo.path.to_string_lossy().into_owned(),
        git: repo.git_status.as_ref().map(git_status_to_json),
        session: repo.session.as_ref().map(session_to_json),
        error: repo.error.clone(),
    }
}

pub fn git_status_to_json(s: &GitStatus) -> JsonGitStatus {
    JsonGitStatus {
        branch: s.branch.clone(),
        detached_head: s.detached_head,
        commit_hash: s.commit_hash.clone(),
        commit_subject: s.commit_subject.clone(),
        commit_time_relative: s.commit_time_relative.clone(),
        staged: s.staged,
        modified: s.modified,
        untracked: s.untracked,
        conflicted: s.conflicted,
        deleted: s.deleted,
        upstream: s.upstream.clone(),
        ahead: s.ahead,
        behind: s.behind,
        status: s.category.display_name().to_string(),
        sync_stale: s.sync_stale,
    }
}

pub fn session_to_json(s: &SessionMemory) -> JsonSession {
    JsonSession {
        branch: s.branch.clone(),
        last_session: s.last_session.map(|t| t.to_rfc3339()),
        note: s.note.clone(),
        next_task: s.next_task.clone(),
        checkpoint_hash: s.checkpoint_hash.clone(),
        checkpoint_time: s.checkpoint_time.map(|t| t.to_rfc3339()),
        checkpoint_summary: s.checkpoint_summary.clone(),
    }
}

pub fn resume_to_json(repo: &Repository, session: &SessionMemory) -> JsonResumeOutput {
    JsonResumeOutput {
        name: repo.name.clone(),
        path: repo.path.to_string_lossy().into_owned(),
        git: repo.git_status.as_ref().map(git_status_to_json),
        session: Some(session_to_json(session)),
    }
}

/// Emit JSON to stdout, diagnostics to stderr.
pub fn emit_json<T: serde::Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    fn make_test_repo() -> Repository {
        Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            git_status: Some(GitStatus {
                branch: Some("main".to_string()),
                detached_head: false,
                commit_hash: Some("abc1234".to_string()),
                commit_subject: Some("Initial commit".to_string()),
                commit_time_relative: Some("2 hours ago".to_string()),
                staged: 0,
                modified: 0,
                untracked: 0,
                conflicted: 0,
                deleted: 0,
                upstream: Some("origin/main".to_string()),
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
    fn test_json_output_valid() {
        let repo = make_test_repo();
        let workspace = repos_to_json(&[repo]);
        let json = serde_json::to_string(&workspace).unwrap();
        // Must be parseable
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["repositories"].is_array());
        assert_eq!(parsed["summary"]["total"], 1);
    }

    #[test]
    fn test_json_stable_fields() {
        let repo = make_test_repo();
        let json_repo = repo_to_json(&repo);
        let json = serde_json::to_string(&json_repo).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("path").is_some());
        assert!(parsed.get("git").is_some());
    }

    #[test]
    fn test_json_handles_none_fields() {
        let repo = Repository {
            name: "empty".to_string(),
            path: PathBuf::from("/tmp/empty"),
            git_status: None,
            session: None,
            error: Some("Permission denied".to_string()),
        };
        let json_repo = repo_to_json(&repo);
        let json = serde_json::to_string(&json_repo).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["git"].is_null());
        assert_eq!(parsed["error"], "Permission denied");
    }
}
