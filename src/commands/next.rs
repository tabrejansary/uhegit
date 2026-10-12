use crate::git::status::collect_status;
use crate::storage::config::Config;
use crate::storage::state::{find_repo_key, load_state, save_next_task, save_state};
use anyhow::Result;

pub fn run_next(repo_name: &str, text: &str, _color: bool, config: &Config) -> Result<()> {
    let mut state = load_state()?;
    let key = find_repo_key(&state, repo_name)?;
    let path = state.repos[&key].path.clone();

    let branch = collect_status(&path, config.git_timeout_secs)
        .ok()
        .and_then(|s| {
            if s.detached_head {
                s.commit_hash.map(|h| format!("(detached:{})", h))
            } else {
                s.branch
            }
        })
        .unwrap_or_else(|| "main".to_string());

    save_next_task(&mut state, repo_name, &branch, text)?;
    save_state(&state)?;

    if text.is_empty() {
        println!(
            "  ✓ Next task cleared for '{}' on branch '{}'.",
            repo_name, branch
        );
    } else {
        println!(
            "  ✓ Next task saved for '{}' on branch '{}':",
            repo_name, branch
        );
        println!("    {}", text);
    }

    Ok(())
}
