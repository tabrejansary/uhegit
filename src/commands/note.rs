use crate::git::status::collect_status;
use crate::storage::config::Config;
use crate::storage::state::{find_repo_key, load_state, save_note, save_state};
use anyhow::Result;

pub fn run_note(repo_name: &str, text: &str, _color: bool, config: &Config) -> Result<()> {
    if text.is_empty() {
        // Clear the note
        let mut state = load_state()?;
        let key = find_repo_key(&state, repo_name)?;
        let path = state.repos[&key].path.clone();

        // Get current branch
        let branch = current_branch(&path, config);
        save_note(&mut state, repo_name, &branch, "")?;
        save_state(&state)?;
        println!(
            "  ✓ Note cleared for '{}' on branch '{}'.",
            repo_name, branch
        );
        return Ok(());
    }

    let mut state = load_state()?;
    let key = find_repo_key(&state, repo_name)?;
    let path = state.repos[&key].path.clone();

    let branch = current_branch(&path, config);
    save_note(&mut state, repo_name, &branch, text)?;
    save_state(&state)?;

    println!("  ✓ Note saved for '{}' on branch '{}':", repo_name, branch);
    println!("    {}", text);

    Ok(())
}

fn current_branch(path: &std::path::Path, config: &Config) -> String {
    collect_status(path, config.git_timeout_secs)
        .ok()
        .and_then(|s| {
            if s.detached_head {
                s.commit_hash.map(|h| format!("(detached:{})", h))
            } else {
                s.branch
            }
        })
        .unwrap_or_else(|| "main".to_string())
}
