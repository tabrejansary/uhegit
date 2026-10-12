use crate::git::executor::GIT_TIMEOUT_SECS;
use crate::git::status::get_toplevel;
use crate::git::status::is_git_repo;
use crate::storage::state::{add_repo, load_state, save_state};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run_add(path_str: &str, _color: bool) -> Result<()> {
    let path = PathBuf::from(path_str);
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("Path does not exist or is inaccessible: {}", path.display()))?;

    if !is_git_repo(&canonical, GIT_TIMEOUT_SECS) {
        return Err(anyhow::anyhow!(
            "'{}' is not a Git repository. Initialize it with 'git init' first.",
            canonical.display()
        ));
    }

    let toplevel = get_toplevel(&canonical, GIT_TIMEOUT_SECS).with_context(|| {
        format!(
            "Could not determine repository root for: {}",
            canonical.display()
        )
    })?;

    let mut state = load_state()?;
    let was_new = add_repo(&mut state, &toplevel, true);
    save_state(&state)?;

    if was_new {
        println!("  ✓ Added: {}", toplevel.display());
    } else {
        println!("  Already tracked: {}", toplevel.display());
    }

    Ok(())
}
