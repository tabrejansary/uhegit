use crate::error::UheGitError;
use crate::storage::config::{atomic_write, data_dir};
use crate::types::SessionMemory;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistent application state: tracked repos and per-repo session memory.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    /// Map of canonical repo path -> tracked repo entry
    pub repos: HashMap<String, TrackedRepo>,
    /// Last time discovery was run
    pub last_discovery: Option<chrono::DateTime<Utc>>,
}

/// A tracked repository entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TrackedRepo {
    /// Canonical absolute path to the working tree root
    pub path: PathBuf,
    /// Whether this repo was added manually (vs discovered)
    pub manually_added: bool,
    /// Whether this repo is marked as inaccessible
    pub inaccessible: bool,
    /// Branch -> SessionMemory map
    pub sessions: HashMap<String, SessionMemory>,
}

impl TrackedRepo {
    pub fn new(path: PathBuf) -> Self {
        TrackedRepo {
            path,
            manually_added: false,
            inaccessible: false,
            sessions: HashMap::new(),
        }
    }

    /// Get repo display name (last path component).
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Get session memory for a branch (or create empty).
    pub fn session(&self, branch: &str) -> SessionMemory {
        self.sessions.get(branch).cloned().unwrap_or_default()
    }

    /// Update session memory for a branch.
    pub fn update_session(&mut self, branch: &str, session: SessionMemory) {
        self.sessions.insert(branch.to_string(), session);
    }

    /// Get the best available session (current branch, or most recent).
    pub fn best_session(&self, current_branch: Option<&str>) -> Option<&SessionMemory> {
        if let Some(branch) = current_branch {
            if let Some(s) = self.sessions.get(branch) {
                return Some(s);
            }
        }
        // Return most recent session across branches
        self.sessions
            .values()
            .filter(|s| s.last_session.is_some())
            .max_by_key(|s| s.last_session)
    }
}

/// Returns the path to the state file.
pub fn state_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("state.json"))
}

/// Load application state from disk, returning empty state if not present.
pub fn load_state() -> Result<AppState> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(AppState::default());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;

    match serde_json::from_str::<AppState>(&content) {
        Ok(state) => Ok(state),
        Err(e) => {
            // Warn but don't fail — return empty state with recovery message
            eprintln!(
                "Warning: State file is corrupted ({}). Starting with empty state. File: {}",
                e,
                path.display()
            );
            Ok(AppState::default())
        }
    }
}

/// Save application state to disk atomically.
pub fn save_state(state: &AppState) -> Result<()> {
    let path = state_path()?;
    let json = serde_json::to_string_pretty(state).context("Failed to serialize state")?;
    atomic_write(&path, json.as_bytes())?;
    Ok(())
}

/// Add a repository path to tracking. Returns true if newly added.
pub fn add_repo(state: &mut AppState, path: &Path, manually: bool) -> bool {
    let key = path_key(path);
    if state.repos.contains_key(&key) {
        // Update inaccessible status if it was previously inaccessible
        if let Some(repo) = state.repos.get_mut(&key) {
            repo.inaccessible = false;
        }
        return false;
    }
    let mut repo = TrackedRepo::new(path.to_path_buf());
    repo.manually_added = manually;
    state.repos.insert(key, repo);
    true
}

/// Remove a repository from tracking (never deletes files).
pub fn remove_repo(state: &mut AppState, name_or_path: &str) -> Result<String> {
    let key = find_repo_key(state, name_or_path)?;
    state.repos.remove(&key);
    Ok(key)
}

/// Find a repo by name or path prefix, returning its canonical key.
pub fn find_repo_key(state: &AppState, name_or_path: &str) -> Result<String> {
    // Try exact path match
    let as_path = PathBuf::from(name_or_path);
    let canonical = std::fs::canonicalize(&as_path).unwrap_or(as_path.clone());
    let canon_key = path_key(&canonical);

    if state.repos.contains_key(&canon_key) {
        return Ok(canon_key);
    }

    // Try name match (case-insensitive on Windows, case-sensitive elsewhere)
    let matches: Vec<(&String, &TrackedRepo)> = state
        .repos
        .iter()
        .filter(|(_, repo)| {
            let repo_name = repo.name();
            #[cfg(target_os = "windows")]
            {
                repo_name.eq_ignore_ascii_case(name_or_path)
            }
            #[cfg(not(target_os = "windows"))]
            {
                repo_name == name_or_path
            }
        })
        .collect();

    match matches.len() {
        0 => Err(UheGitError::RepoNotFound {
            name: name_or_path.to_string(),
        }
        .into()),
        1 => Ok(matches[0].0.clone()),
        _ => {
            let names: Vec<&str> = matches
                .iter()
                .map(|(_, r)| r.path.to_str().unwrap_or("?"))
                .collect();
            Err(anyhow::anyhow!(
                "Ambiguous repository name '{}'. Matches:\n  {}",
                name_or_path,
                names.join("\n  ")
            ))
        }
    }
}

/// Get a mutable reference to a repo by name or path.
pub fn get_repo_mut<'a>(
    state: &'a mut AppState,
    name_or_path: &str,
) -> Result<&'a mut TrackedRepo> {
    let key = find_repo_key(state, name_or_path)?;
    state.repos.get_mut(&key).ok_or_else(|| {
        UheGitError::RepoNotFound {
            name: name_or_path.to_string(),
        }
        .into()
    })
}

/// Get an immutable reference to a repo by name or path.
pub fn get_repo<'a>(state: &'a AppState, name_or_path: &str) -> Result<&'a TrackedRepo> {
    let key = find_repo_key(state, name_or_path)?;
    state.repos.get(&key).ok_or_else(|| {
        UheGitError::RepoNotFound {
            name: name_or_path.to_string(),
        }
        .into()
    })
}

/// Save a note for a repository's current branch.
pub fn save_note(state: &mut AppState, repo_name: &str, branch: &str, note: &str) -> Result<()> {
    if note.is_empty() {
        // Clear the note
        let repo = get_repo_mut(state, repo_name)?;
        repo.sessions.entry(branch.to_string()).and_modify(|s| {
            s.note = None;
        });
        return Ok(());
    }

    let repo = get_repo_mut(state, repo_name)?;
    let mut session = repo.session(branch);
    session.note = Some(note.to_string());
    session.last_session = Some(Utc::now());
    session.branch = Some(branch.to_string());
    repo.update_session(branch, session);
    Ok(())
}

/// Save next task for a repository's current branch.
pub fn save_next_task(
    state: &mut AppState,
    repo_name: &str,
    branch: &str,
    task: &str,
) -> Result<()> {
    let repo = get_repo_mut(state, repo_name)?;
    let mut session = repo.session(branch);
    session.next_task = if task.is_empty() {
        None
    } else {
        Some(task.to_string())
    };
    session.last_session = Some(Utc::now());
    session.branch = Some(branch.to_string());
    repo.update_session(branch, session);
    Ok(())
}

/// Save a checkpoint for a repository.
pub fn save_checkpoint(
    state: &mut AppState,
    repo_name: &str,
    branch: &str,
    commit_hash: &str,
    summary: &str,
) -> Result<()> {
    let repo = get_repo_mut(state, repo_name)?;
    let mut session = repo.session(branch);
    session.checkpoint_hash = Some(commit_hash.to_string());
    session.checkpoint_time = Some(Utc::now());
    session.checkpoint_summary = Some(summary.to_string());
    session.last_session = Some(Utc::now());
    session.branch = Some(branch.to_string());
    repo.update_session(branch, session);
    Ok(())
}

/// Forget session memory for a repo.
pub fn forget_repo_sessions(state: &mut AppState, repo_name: &str) -> Result<()> {
    let repo = get_repo_mut(state, repo_name)?;
    repo.sessions.clear();
    Ok(())
}

/// Stable key for a path (canonical string).
fn path_key(path: &Path) -> String {
    // Normalize separators to forward slash for cross-platform consistency
    path.to_string_lossy().replace('\\', "/")
}

/// Mark inaccessible repos without removing them.
pub fn validate_repos(state: &mut AppState) {
    for repo in state.repos.values_mut() {
        repo.inaccessible = !repo.path.exists();
    }
}

/// Get sorted list of repo paths.
pub fn sorted_repo_paths(state: &AppState) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = state.repos.values().map(|r| r.path.clone()).collect();
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_add_repo() {
        let mut state = AppState::default();
        let dir = make_dir();
        assert!(add_repo(&mut state, dir.path(), false));
        assert!(!add_repo(&mut state, dir.path(), false)); // duplicate
        assert_eq!(state.repos.len(), 1);
    }

    #[test]
    fn test_remove_repo() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let name = TrackedRepo::new(dir.path().to_path_buf()).name();
        remove_repo(&mut state, &name).unwrap();
        assert_eq!(state.repos.len(), 0);
    }

    #[test]
    fn test_save_note() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let name = TrackedRepo::new(dir.path().to_path_buf()).name();
        save_note(&mut state, &name, "main", "Fix the bug").unwrap();
        let repo = get_repo(&state, &name).unwrap();
        assert_eq!(repo.sessions["main"].note.as_deref(), Some("Fix the bug"));
    }

    #[test]
    fn test_save_note_branch_isolation() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let name = TrackedRepo::new(dir.path().to_path_buf()).name();
        save_note(&mut state, &name, "main", "Main note").unwrap();
        save_note(&mut state, &name, "feature/x", "Feature note").unwrap();
        let repo = get_repo(&state, &name).unwrap();
        assert_eq!(repo.sessions["main"].note.as_deref(), Some("Main note"));
        assert_eq!(
            repo.sessions["feature/x"].note.as_deref(),
            Some("Feature note")
        );
    }

    #[test]
    fn test_save_checkpoint() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let name = TrackedRepo::new(dir.path().to_path_buf()).name();
        save_checkpoint(&mut state, &name, "main", "abc1234", "1 modified").unwrap();
        let repo = get_repo(&state, &name).unwrap();
        assert_eq!(
            repo.sessions["main"].checkpoint_hash.as_deref(),
            Some("abc1234")
        );
    }

    #[test]
    fn test_state_serialization() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let json = serde_json::to_string(&state).unwrap();
        let back: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repos.len(), 1);
    }

    #[test]
    fn test_forget_sessions() {
        let mut state = AppState::default();
        let dir = make_dir();
        add_repo(&mut state, dir.path(), false);
        let name = TrackedRepo::new(dir.path().to_path_buf()).name();
        save_note(&mut state, &name, "main", "Some note").unwrap();
        forget_repo_sessions(&mut state, &name).unwrap();
        let repo = get_repo(&state, &name).unwrap();
        assert!(repo.sessions.is_empty());
    }
}
