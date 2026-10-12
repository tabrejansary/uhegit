mod add;
mod checkpoint;
mod completion;
mod config;
mod forget;
mod next;
mod note;
mod path_cmd;
mod remove;
mod repos;
mod resume;
mod scan;
mod status;
mod version;

use crate::cli::{Cli, Commands, ConfigAction};
use crate::discovery::scanner::default_discovery_roots;
use crate::git::executor::check_git_available;
use crate::output::display::use_color;
use crate::storage::config::load_config;
use crate::storage::state::{add_repo, load_state, save_state};
use crate::types::StatusFilter;
use anyhow::Result;
use chrono::{Duration, Utc};
use std::path::PathBuf;

/// Main command dispatch.
pub fn dispatch(cli: Cli) -> Result<()> {
    // Git availability check (skip for completion and version)
    let skip_git_check = matches!(
        &cli.command,
        Some(Commands::Completion { .. }) | Some(Commands::Version) | Some(Commands::Config { .. })
    );

    if !skip_git_check {
        if let Err(e) = check_git_available() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    let config = load_config().unwrap_or_default();
    let color = use_color(config.color);

    // Parse global filters from top-level flags
    let json = cli.json;
    let filters = build_filters(
        &cli.dirty,
        &cli.unpushed,
        &cli.behind,
        &cli.conflicts,
        &cli.clean,
        &cli.stale,
    )?;

    match cli.command {
        None => {
            // Default command: workspace overview
            ensure_initial_discovery(&config)?;
            status::run_status(json, filters, color, &config)
        }
        Some(Commands::Status {
            json: j,
            dirty,
            unpushed,
            behind,
            conflicts,
            clean,
            stale,
        }) => {
            let json = json || j;
            let filters = build_filters(&dirty, &unpushed, &behind, &conflicts, &clean, &stale)?;
            ensure_initial_discovery(&config)?;
            status::run_status(json, filters, color, &config)
        }
        Some(Commands::Scan { home, path, depth }) => {
            scan::run_scan(home, path, depth, color, &config)
        }
        Some(Commands::Add { path }) => add::run_add(&path, color),
        Some(Commands::Remove { repository }) => remove::run_remove(&repository, color),
        Some(Commands::Repos { json: j }) => {
            let json = json || j;
            repos::run_repos(json, color, &config)
        }
        Some(Commands::Note { repository, text }) => {
            note::run_note(&repository, &text, color, &config)
        }
        Some(Commands::Next { repository, text }) => {
            next::run_next(&repository, &text, color, &config)
        }
        Some(Commands::Checkpoint { repository }) => {
            checkpoint::run_checkpoint(&repository, color, &config)
        }
        Some(Commands::Resume {
            repository,
            json: j,
        }) => {
            let json = json || j;
            resume::run_resume(&repository, json, color, &config)
        }
        Some(Commands::Path { repository }) => path_cmd::run_path(&repository),
        Some(Commands::Forget { repository }) => forget::run_forget(&repository, color),
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => config::run_show(),
            ConfigAction::Path => config::run_path(),
            ConfigAction::Set { key, value } => config::run_set(&key, &value),
        },
        Some(Commands::Version) => {
            version::run_version();
            Ok(())
        }
        Some(Commands::Completion { shell }) => {
            completion::run_completion(shell);
            Ok(())
        }
    }
}

fn build_filters(
    dirty: &bool,
    unpushed: &bool,
    behind: &bool,
    conflicts: &bool,
    clean: &bool,
    stale: &Option<String>,
) -> Result<Vec<StatusFilter>> {
    let mut filters = Vec::new();
    if *dirty {
        filters.push(StatusFilter::Dirty);
    }
    if *unpushed {
        filters.push(StatusFilter::Unpushed);
    }
    if *behind {
        filters.push(StatusFilter::Behind);
    }
    if *conflicts {
        filters.push(StatusFilter::Conflicts);
    }
    if *clean {
        filters.push(StatusFilter::Clean);
    }
    if let Some(stale_str) = stale {
        let days = parse_duration_days(stale_str)?;
        filters.push(StatusFilter::Stale(days));
    }
    Ok(filters)
}

fn parse_duration_days(s: &str) -> Result<u64> {
    let s = s.trim();
    if let Some(days_str) = s.strip_suffix('d') {
        days_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid duration: '{}'. Use format like '7d'.", s))
    } else if let Some(weeks_str) = s.strip_suffix('w') {
        weeks_str
            .parse::<u64>()
            .map(|w| w * 7)
            .map_err(|_| anyhow::anyhow!("Invalid duration: '{}'. Use format like '2w'.", s))
    } else {
        s.parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid duration: '{}'. Use format like '7d'.", s))
    }
}

/// On first run (no repos registered), automatically scan default roots.
fn ensure_initial_discovery(config: &crate::storage::config::Config) -> Result<()> {
    let state = load_state()?;
    if state.repos.is_empty() || should_rediscover(&state) {
        // Run discovery silently
        let roots = get_discovery_roots(config);
        if roots.is_empty() {
            return Ok(());
        }
        let mut state = state;
        let known = state
            .repos
            .values()
            .map(|r| r.path.clone())
            .collect::<std::collections::HashSet<_>>();

        for root in &roots {
            if !root.exists() {
                continue;
            }
            let mut result = crate::discovery::scanner::ScanResult::default();
            crate::discovery::scanner::scan_for_repos(
                root,
                config.max_depth,
                config.git_timeout_secs,
                &mut result,
                &known,
            );
            for path in result.found {
                add_repo(&mut state, &path, false);
            }
        }
        state.last_discovery = Some(Utc::now());
        save_state(&state)?;
    }
    Ok(())
}

fn should_rediscover(state: &crate::storage::state::AppState) -> bool {
    // Re-discover if no discovery has run in 24h
    match state.last_discovery {
        None => true,
        Some(last) => Utc::now() - last > Duration::hours(24),
    }
}

pub fn get_discovery_roots(config: &crate::storage::config::Config) -> Vec<PathBuf> {
    let mut roots = default_discovery_roots();
    roots.extend(config.discovery_roots.clone());
    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    roots.retain(|p| seen.insert(p.clone()));
    roots
}
