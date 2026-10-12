use crate::git::status::collect_status;
use crate::storage::config::Config;
use crate::storage::state::{find_repo_key, load_state, save_checkpoint, save_state};
use anyhow::Result;

pub fn run_checkpoint(repo_name: &str, _color: bool, config: &Config) -> Result<()> {
    let mut state = load_state()?;
    let key = find_repo_key(&state, repo_name)?;
    let path = state.repos[&key].path.clone();

    let git_status = collect_status(&path, config.git_timeout_secs)?;

    let branch = if git_status.detached_head {
        git_status
            .commit_hash
            .as_ref()
            .map(|h| format!("(detached:{})", h))
            .unwrap_or_else(|| "(detached)".to_string())
    } else {
        git_status
            .branch
            .clone()
            .unwrap_or_else(|| "main".to_string())
    };

    let commit_hash = git_status.commit_hash.clone().unwrap_or_default();

    // Build a compact working-tree summary
    let mut parts = Vec::new();
    if git_status.conflicted > 0 {
        parts.push(format!("{} conflicted", git_status.conflicted));
    }
    if git_status.staged > 0 {
        parts.push(format!("{} staged", git_status.staged));
    }
    if git_status.modified > 0 {
        parts.push(format!("{} modified", git_status.modified));
    }
    if git_status.deleted > 0 {
        parts.push(format!("{} deleted", git_status.deleted));
    }
    if git_status.untracked > 0 {
        parts.push(format!("{} untracked", git_status.untracked));
    }
    let summary = if parts.is_empty() {
        "Clean working tree".to_string()
    } else {
        parts.join(", ")
    };

    save_checkpoint(&mut state, repo_name, &branch, &commit_hash, &summary)?;
    save_state(&state)?;

    println!(
        "\n  ✓ Checkpoint saved for '{}' on branch '{}'",
        repo_name, branch
    );
    println!(
        "    Commit:  {}",
        if commit_hash.is_empty() {
            "(no commits)".to_string()
        } else {
            commit_hash
        }
    );
    println!("    Status:  {}", summary);
    println!("    Note: Checkpoint records state only. Changes are NOT backed up.\n");

    Ok(())
}
