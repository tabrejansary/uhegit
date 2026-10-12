use crate::commands::status::collect_all_repos;
use crate::output::json::{emit_json, repos_to_json};
use crate::storage::config::Config;
use crate::storage::state::{load_state, sorted_repo_paths, validate_repos};
use anyhow::Result;

pub fn run_repos(json: bool, _color: bool, config: &Config) -> Result<()> {
    let mut state = load_state()?;
    validate_repos(&mut state);

    let paths = sorted_repo_paths(&state);

    if json {
        // Collect full status for JSON output
        let repos = collect_all_repos(&paths, &state, config);
        emit_json(&repos_to_json(&repos))?;
        return Ok(());
    }

    if paths.is_empty() {
        println!("\n  No repositories tracked. Run 'uhegit scan' to discover repositories.\n");
        return Ok(());
    }

    println!("\n  Tracked repositories ({})\n", paths.len());
    for (i, path) in paths.iter().enumerate() {
        let repo = state.repos.values().find(|r| r.path == *path);
        let inaccessible = repo.map(|r| r.inaccessible).unwrap_or(false);
        let manually = repo.map(|r| r.manually_added).unwrap_or(false);
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let flags = [
            if inaccessible {
                Some("[inaccessible]")
            } else {
                None
            },
            if manually { Some("[manual]") } else { None },
        ]
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

        println!("  {:3}. {:<30} {}  {}", i + 1, name, path.display(), flags);
    }
    println!();

    Ok(())
}
