use crate::git::status::collect_status;
use crate::output::display::print_resume_output;
use crate::output::json::{emit_json, resume_to_json};
use crate::storage::config::Config;
use crate::storage::state::{find_repo_key, get_repo, load_state};
use crate::types::Repository;
use anyhow::Result;

pub fn run_resume(repo_name: &str, json: bool, color: bool, config: &Config) -> Result<()> {
    let state = load_state()?;
    let _key = find_repo_key(&state, repo_name)?;
    let tracked = get_repo(&state, repo_name)?;
    let path = tracked.path.clone();

    let git_status = collect_status(&path, config.git_timeout_secs).ok();

    let branch = git_status.as_ref().and_then(|s| {
        if s.detached_head {
            s.commit_hash.as_ref().map(|h| format!("(detached:{})", h))
        } else {
            s.branch.clone()
        }
    });

    // Get best available session
    let session = branch
        .as_ref()
        .and_then(|b| tracked.sessions.get(b))
        .cloned()
        .or_else(|| tracked.best_session(branch.as_deref()).cloned())
        .unwrap_or_default();

    let repo = Repository {
        name: tracked.name(),
        path: path.clone(),
        git_status,
        session: Some(session.clone()),
        error: None,
    };

    if json {
        let output = resume_to_json(&repo, &session);
        emit_json(&output)?;
    } else {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        print_resume_output(&repo, &session, color, &mut out);
    }

    Ok(())
}
