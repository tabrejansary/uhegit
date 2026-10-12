use crate::storage::state::{get_repo, load_state};
use anyhow::Result;

pub fn run_path(repo_name: &str) -> Result<()> {
    let state = load_state()?;
    let repo = get_repo(&state, repo_name)?;
    // Print only the path, suitable for scripting
    println!("{}", repo.path.display());
    Ok(())
}
