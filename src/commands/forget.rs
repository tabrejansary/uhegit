use crate::storage::state::{find_repo_key, forget_repo_sessions, load_state, save_state};
use anyhow::Result;
use std::io::{self, Write};

pub fn run_forget(repo_name: &str, _color: bool) -> Result<()> {
    let mut state = load_state()?;
    let key = find_repo_key(&state, repo_name)?;
    let name = state.repos[&key].name();
    let session_count = state.repos[&key].sessions.len();

    if session_count == 0 {
        println!("  No session data found for '{}'.", name);
        return Ok(());
    }

    // Require explicit confirmation
    print!(
        "\n  This will remove {} session record(s) for '{}'.\n  Repository files will NOT be deleted.\n  Confirm? [y/N] ",
        session_count,
        name
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        forget_repo_sessions(&mut state, repo_name)?;
        save_state(&state)?;
        println!("  ✓ Session memory cleared for '{}'.", name);
    } else {
        println!("  Cancelled. No changes made.");
    }

    Ok(())
}
