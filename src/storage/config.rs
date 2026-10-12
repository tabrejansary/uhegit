use crate::error::UheGitError;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// UheGit application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Extra discovery roots (in addition to auto-detected defaults)
    pub discovery_roots: Vec<PathBuf>,
    /// Maximum directory scan depth
    pub max_depth: usize,
    /// Maximum concurrent Git workers
    pub max_workers: usize,
    /// Git command timeout in seconds
    pub git_timeout_secs: u64,
    /// Whether to use terminal colors
    pub color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            discovery_roots: Vec::new(),
            max_depth: 6,
            max_workers: 8,
            git_timeout_secs: 15,
            color: true,
        }
    }
}

/// Returns the OS-appropriate project directories for UheGit.
pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "uhegit", "uhegit").ok_or_else(|| {
        UheGitError::Config("Could not determine application data directory".to_string()).into()
    })
}

/// Returns the path to the configuration file.
pub fn config_path() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    Ok(dirs.config_dir().join("config.json"))
}

/// Returns the path to the data directory.
pub fn data_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    Ok(dirs.data_dir().to_path_buf())
}

/// Load configuration from disk, returning defaults if not present or corrupt.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    serde_json::from_str(&content).map_err(|e| {
        UheGitError::Config(format!(
            "Config file is malformed: {}. Run 'uhegit config show' to inspect, or delete {:?} to reset.",
            e,
            path
        ))
        .into()
    })
}

/// Save configuration to disk atomically.
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    ensure_parent_dir(&path)?;

    let json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    atomic_write(&path, json.as_bytes())?;
    Ok(())
}

/// Load configuration, set a key-value pair, save.
pub fn set_config_value(key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;
    match key {
        "max-depth" => {
            config.max_depth = value.parse::<usize>().map_err(|_| {
                UheGitError::Config(format!("Invalid value for max-depth: '{}'", value))
            })?;
        }
        "max-workers" => {
            config.max_workers = value.parse::<usize>().map_err(|_| {
                UheGitError::Config(format!("Invalid value for max-workers: '{}'", value))
            })?;
        }
        "git-timeout" => {
            config.git_timeout_secs = value.parse::<u64>().map_err(|_| {
                UheGitError::Config(format!("Invalid value for git-timeout: '{}'", value))
            })?;
        }
        "color" => {
            config.color = match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    return Err(UheGitError::Config(format!(
                        "Invalid value for color: '{}'. Use true/false.",
                        value
                    ))
                    .into())
                }
            };
        }
        _ => {
            return Err(UheGitError::Config(format!(
                "Unknown config key: '{}'. Valid keys: max-depth, max-workers, git-timeout, color",
                key
            ))
            .into());
        }
    }
    save_config(&config)
}

/// Atomic write: write to a temp file, then rename over the target.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    ensure_parent_dir(path)?;
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp = tempfile::NamedTempFile::new_in(parent)
        .context("Failed to create temporary file for atomic write")?;
    std::io::Write::write_all(&mut tmp.as_file(), data)
        .context("Failed to write to temporary file")?;
    tmp.persist(path)
        .map_err(|e| anyhow::anyhow!("Failed to persist file {}: {}", path.display(), e))?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_depth, 6);
        assert_eq!(config.max_workers, 8);
        assert!(config.color);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_depth, config.max_depth);
    }

    #[test]
    fn test_atomic_write() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        atomic_write(&path, b"hello").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_atomic_write_overwrites() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        atomic_write(&path, b"first").unwrap();
        atomic_write(&path, b"second").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "second");
    }
}
