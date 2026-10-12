use crate::git::status::collect_status;
use crate::output::display::print_workspace_overview;
use crate::output::json::{emit_json, repos_to_json};
use crate::storage::config::Config;
use crate::storage::state::{load_state, sorted_repo_paths, validate_repos};
use crate::types::{Repository, StatusFilter};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn run_status(
    json: bool,
    filters: Vec<StatusFilter>,
    color: bool,
    config: &Config,
) -> Result<()> {
    let mut state = load_state()?;
    validate_repos(&mut state);

    let paths = sorted_repo_paths(&state);
    if paths.is_empty() {
        if json {
            let ws = repos_to_json(&[]);
            emit_json(&ws)?;
        } else {
            println!("\n  UheGit — Your Workspace\n");
            println!("  No repositories found.");
            println!("  Run 'uhegit scan' to discover your Git repositories.\n");
        }
        return Ok(());
    }

    let repos = collect_all_repos(&paths, &state, config);
    let mut repos = apply_filters(repos, &filters);
    sort_repos(&mut repos);

    if json {
        let ws = repos_to_json(&repos);
        emit_json(&ws)?;
    } else {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        print_workspace_overview(&repos, color, &mut out);

        if !filters.is_empty() && repos.is_empty() {
            println!("  No repositories match the applied filter.");
        }
    }

    Ok(())
}

/// Collect status for all tracked repos in parallel.
pub fn collect_all_repos(
    paths: &[PathBuf],
    state: &crate::storage::state::AppState,
    config: &Config,
) -> Vec<Repository> {
    let timeout = config.git_timeout_secs;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.max_workers)
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    let results: Vec<Repository> = pool.install(|| {
        paths
            .par_iter()
            .map(|path| collect_single_repo(path, state, timeout))
            .collect()
    });

    results
}

fn collect_single_repo(
    path: &Path,
    state: &crate::storage::state::AppState,
    timeout: u64,
) -> Repository {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if !path.exists() {
        return Repository {
            name,
            path: path.to_path_buf(),
            git_status: None,
            session: None,
            error: Some("Path no longer exists".to_string()),
        };
    }

    match collect_status(path, timeout) {
        Ok(git_status) => {
            // Find matching repo in state to get session
            let session = find_session(state, path, git_status.branch.as_deref());
            Repository {
                name,
                path: path.to_path_buf(),
                git_status: Some(git_status),
                session,
                error: None,
            }
        }
        Err(e) => Repository {
            name,
            path: path.to_path_buf(),
            git_status: None,
            session: find_session(state, path, None),
            error: Some(e.to_string()),
        },
    }
}

fn find_session(
    state: &crate::storage::state::AppState,
    path: &Path,
    branch: Option<&str>,
) -> Option<crate::types::SessionMemory> {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let repo = state.repos.values().find(|r| {
        let rp = r.path.to_string_lossy().replace('\\', "/");
        rp == path_str
    })?;

    let session = if let Some(b) = branch {
        repo.sessions.get(b).cloned()
    } else {
        None
    };

    // If no branch-specific session, get best available
    session.or_else(|| repo.best_session(branch).cloned())
}

fn apply_filters(repos: Vec<Repository>, filters: &[StatusFilter]) -> Vec<Repository> {
    if filters.is_empty() {
        return repos;
    }
    repos
        .into_iter()
        .filter(|r| filters.iter().all(|f| f.matches(r)))
        .collect()
}

fn sort_repos(repos: &mut [Repository]) {
    repos.sort_by(|a, b| {
        // Sort by category priority first, then alphabetically by name
        let cat_a = a
            .git_status
            .as_ref()
            .map(|s| &s.category)
            .map(|c| c.clone() as u8)
            .unwrap_or(6);
        let cat_b = b
            .git_status
            .as_ref()
            .map(|s| &s.category)
            .map(|c| c.clone() as u8)
            .unwrap_or(6);
        cat_a.cmp(&cat_b).then_with(|| a.name.cmp(&b.name))
    });
}
