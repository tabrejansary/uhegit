use crate::commands::get_discovery_roots;
use crate::discovery::scanner::{home_dir, scan_for_repos, ScanResult};
use crate::output::display::print_scan_result;
use crate::storage::config::Config;
use crate::storage::state::{add_repo, load_state, save_state};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn run_scan(
    home: bool,
    path: Option<String>,
    depth: usize,
    color: bool,
    config: &Config,
) -> Result<()> {
    let mut state = load_state()?;
    let timeout = config.git_timeout_secs;

    let roots: Vec<PathBuf> = if home {
        match home_dir() {
            Some(h) => {
                println!(
                    "\n  Scanning home directory: {}\n  This may take a while...\n",
                    h.display()
                );
                vec![h]
            }
            None => {
                eprintln!("Could not determine home directory.");
                return Ok(());
            }
        }
    } else if let Some(p) = path {
        let p = PathBuf::from(&p);
        if !p.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", p.display()));
        }
        vec![p]
    } else {
        let roots = get_discovery_roots(config);
        if roots.is_empty() {
            println!("\n  No discovery roots configured.");
            println!("  Try 'uhegit scan --home' or 'uhegit scan --path <dir>'.\n");
            return Ok(());
        }
        println!("\n  Scanning {} root(s)...\n", roots.len());
        roots
    };

    let known: HashSet<PathBuf> = state.repos.values().map(|r| r.path.clone()).collect();
    let mut total_result = ScanResult::default();

    for root in &roots {
        if !root.exists() {
            continue;
        }
        let mut result = ScanResult::default();
        scan_for_repos(root, depth, timeout, &mut result, &known);

        for p in &result.found {
            let newly = add_repo(&mut state, p, false);
            if newly {
                total_result.newly_added += 1;
            } else {
                total_result.already_known += 1;
            }
        }
        total_result.found.extend(result.found);
        total_result.skipped += result.skipped;
        total_result.inaccessible += result.inaccessible;
    }

    state.last_discovery = Some(Utc::now());
    save_state(&state).context("Failed to save state after scan")?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    print_scan_result(
        total_result.found.len(),
        total_result.newly_added,
        total_result.already_known,
        total_result.skipped,
        total_result.inaccessible,
        color,
        &mut out,
    );

    Ok(())
}
