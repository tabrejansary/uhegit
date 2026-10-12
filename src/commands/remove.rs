use crate::storage::state::{find_repo_key, load_state, save_state};
use anyhow::Result;

pub fn run_remove(repo: &str, _color: bool) -> Result<()> {
    let mut state = load_state()?;
    let key = find_repo_key(&state, repo)?;
    let path = state.repos[&key].path.display().to_string();
    state.repos.remove(&key);
    save_state(&state)?;
    println!(
        "  ✓ Removed '{}' from tracking. Repository files were NOT deleted.",
        path
    );
    println!("    Path: {}", path);
    Ok(())
}
