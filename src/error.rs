use thiserror::Error;

#[derive(Debug, Error)]
pub enum UheGitError {
    #[error("Git executable not found. Please install Git and ensure it is in your PATH.")]
    GitNotFound,

    #[error("Git command timed out after {seconds}s in: {path}")]
    GitTimeout { seconds: u64, path: String },

    #[error("Git command failed in {path}: {message}")]
    GitCommandFailed { path: String, message: String },

    #[error("Repository not found: {name}")]
    RepoNotFound { name: String },

    #[error("Directory is not a Git repository: {path}")]
    NotARepository { path: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Note cannot be empty")]
    EmptyNote,

    #[error("Path does not exist: {path}")]
    PathNotFound { path: String },

    #[error("Incompatible filters: {0}")]
    IncompatibleFilters(String),
}
