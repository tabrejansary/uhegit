use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single discovered Git repository and its current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Short display name (last path component)
    pub name: String,
    /// Canonical absolute path to the working tree root
    pub path: PathBuf,
    /// Git status information (None if collection failed)
    pub git_status: Option<GitStatus>,
    /// Saved work session memory for this repo+branch
    pub session: Option<SessionMemory>,
    /// Error encountered during status collection, if any
    pub error: Option<String>,
}

/// The current Git state of a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    /// Current branch name, or None if detached HEAD
    pub branch: Option<String>,
    /// True if in detached HEAD state
    pub detached_head: bool,
    /// OID of the current commit (short)
    pub commit_hash: Option<String>,
    /// Subject line of the latest commit
    pub commit_subject: Option<String>,
    /// Relative time of the latest commit (e.g. "2 hours ago")
    pub commit_time_relative: Option<String>,
    /// Number of staged files
    pub staged: u32,
    /// Number of modified (unstaged) tracked files
    pub modified: u32,
    /// Number of untracked files
    pub untracked: u32,
    /// Number of conflicted files
    pub conflicted: u32,
    /// Number of deleted files
    pub deleted: u32,
    /// Upstream tracking branch, if any
    pub upstream: Option<String>,
    /// Commits ahead of upstream
    pub ahead: Option<u32>,
    /// Commits behind upstream
    pub behind: Option<u32>,
    /// Computed overall status category
    pub category: StatusCategory,
    /// True if remote sync data may be stale (based on local tracking refs only)
    pub sync_stale: bool,
}

/// High-level status category for a repository.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StatusCategory {
    /// Has merge conflicts or unresolvable state
    Conflict = 0,
    /// Dirty working tree (staged/modified/untracked/deleted files)
    Dirty = 1,
    /// Commits both ahead and behind upstream
    Diverged = 2,
    /// Commits ahead of upstream only
    Unpushed = 3,
    /// Commits behind upstream only
    Behind = 4,
    /// Clean working tree, synced
    Clean = 5,
    /// Could not determine status
    Unknown = 6,
}

impl StatusCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            StatusCategory::Conflict => "Conflicts",
            StatusCategory::Dirty => "Uncommitted changes",
            StatusCategory::Diverged => "Diverged",
            StatusCategory::Unpushed => "Unpushed commits",
            StatusCategory::Behind => "Behind upstream",
            StatusCategory::Clean => "Clean",
            StatusCategory::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for StatusCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Per-repository, per-branch work session memory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMemory {
    /// Last session timestamp
    pub last_session: Option<DateTime<Utc>>,
    /// Branch this session was recorded on
    pub branch: Option<String>,
    /// User-written note
    pub note: Option<String>,
    /// User-written next task
    pub next_task: Option<String>,
    /// Commit hash at time of last checkpoint
    pub checkpoint_hash: Option<String>,
    /// Timestamp of last checkpoint
    pub checkpoint_time: Option<DateTime<Utc>>,
    /// Working tree summary at last checkpoint
    pub checkpoint_summary: Option<String>,
}

/// Status filter the user may apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusFilter {
    Dirty,
    Unpushed,
    Behind,
    Conflicts,
    Clean,
    Stale(u64), // days
}

impl StatusFilter {
    pub fn matches(&self, repo: &Repository) -> bool {
        match self {
            StatusFilter::Dirty => repo
                .git_status
                .as_ref()
                .map(|s| s.staged > 0 || s.modified > 0 || s.untracked > 0 || s.deleted > 0)
                .unwrap_or(false),
            StatusFilter::Unpushed => repo
                .git_status
                .as_ref()
                .map(|s| s.ahead.unwrap_or(0) > 0)
                .unwrap_or(false),
            StatusFilter::Behind => repo
                .git_status
                .as_ref()
                .map(|s| s.behind.unwrap_or(0) > 0)
                .unwrap_or(false),
            StatusFilter::Conflicts => repo
                .git_status
                .as_ref()
                .map(|s| s.conflicted > 0)
                .unwrap_or(false),
            StatusFilter::Clean => repo
                .git_status
                .as_ref()
                .map(|s| s.category == StatusCategory::Clean)
                .unwrap_or(false),
            StatusFilter::Stale(days) => {
                let threshold = Utc::now() - chrono::Duration::days(*days as i64);
                repo.session
                    .as_ref()
                    .and_then(|s| s.last_session)
                    .map(|t| t < threshold)
                    .unwrap_or(true)
            }
        }
    }
}
