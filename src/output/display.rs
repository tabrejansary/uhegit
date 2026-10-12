use crate::types::{Repository, SessionMemory, StatusCategory};
use crossterm::style::{Color, Stylize};
use std::io::Write;

/// Whether to use color output.
pub fn use_color(config_color: bool) -> bool {
    if !config_color {
        return false;
    }
    // Check NO_COLOR env var
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    // Check if stdout is a terminal
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

pub fn print_workspace_overview(repos: &[Repository], color: bool, writer: &mut impl Write) {
    if repos.is_empty() {
        writeln!(
            writer,
            "No repositories found. Run 'uhegit scan' to discover repositories."
        )
        .ok();
        return;
    }

    // Header
    writeln!(writer).ok();
    if color {
        writeln!(
            writer,
            "{}",
            "  UheGit — Your Workspace".with(Color::Cyan).bold()
        )
        .ok();
        writeln!(writer, "  {}", "─".repeat(70).with(Color::DarkGrey)).ok();
    } else {
        writeln!(writer, "  UheGit — Your Workspace").ok();
        writeln!(writer, "  {}", "─".repeat(70)).ok();
    }
    writeln!(writer).ok();

    // Calculate column widths
    let name_w = repos
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(10)
        .clamp(10, 30);
    let branch_w = repos
        .iter()
        .map(|r| {
            r.git_status
                .as_ref()
                .map(|s| s.branch.as_deref().unwrap_or("(detached)").len())
                .unwrap_or(8)
        })
        .max()
        .unwrap_or(8)
        .clamp(8, 25);
    let status_w = 24usize;

    // Column headers
    if color {
        writeln!(
            writer,
            "  {:<nw$}  {:<bw$}  {:<sw$}  {}",
            "REPOSITORY".with(Color::DarkGrey).bold(),
            "BRANCH".with(Color::DarkGrey).bold(),
            "STATUS".with(Color::DarkGrey).bold(),
            "SYNC".with(Color::DarkGrey).bold(),
            nw = name_w,
            bw = branch_w,
            sw = status_w,
        )
        .ok();
    } else {
        writeln!(
            writer,
            "  {:<nw$}  {:<bw$}  {:<sw$}  SYNC",
            "REPOSITORY",
            "BRANCH",
            "STATUS",
            nw = name_w,
            bw = branch_w,
            sw = status_w,
        )
        .ok();
    }

    for repo in repos {
        print_repo_row(repo, name_w, branch_w, status_w, color, writer);
    }

    writeln!(writer).ok();

    // Summary line
    let total = repos.len();
    let clean_count = repos
        .iter()
        .filter(|r| {
            r.git_status
                .as_ref()
                .map(|s| s.category == StatusCategory::Clean)
                .unwrap_or(false)
        })
        .count();
    let attention_count = total - clean_count;

    let errors = repos.iter().filter(|r| r.error.is_some()).count();
    let error_note = if errors > 0 {
        format!(" | {} with errors", errors)
    } else {
        String::new()
    };

    if color {
        writeln!(
            writer,
            "  {} repositories | {} clean | {} need attention{}",
            total.to_string().with(Color::White).bold(),
            clean_count.to_string().with(Color::Green),
            attention_count.to_string().with(Color::Yellow),
            error_note
        )
        .ok();
    } else {
        writeln!(
            writer,
            "  {} repositories | {} clean | {} need attention{}",
            total, clean_count, attention_count, error_note
        )
        .ok();
    }

    // Show next tasks
    let has_next_tasks = repos.iter().any(|r| {
        r.session
            .as_ref()
            .and_then(|s| s.next_task.as_ref())
            .is_some()
    });
    if has_next_tasks {
        writeln!(writer).ok();
        if color {
            writeln!(writer, "  {}", "NEXT TASKS".with(Color::DarkGrey).bold()).ok();
        } else {
            writeln!(writer, "  NEXT TASKS").ok();
        }
        for repo in repos {
            if let Some(task) = repo.session.as_ref().and_then(|s| s.next_task.as_ref()) {
                if color {
                    writeln!(
                        writer,
                        "  {}: {}",
                        repo.name.clone().with(Color::Cyan),
                        task
                    )
                    .ok();
                } else {
                    writeln!(writer, "  {}: {}", repo.name, task).ok();
                }
            }
        }
    }

    writeln!(writer).ok();

    // Suggest useful commands
    let has_sessions = repos.iter().any(|r| r.session.is_some());
    if has_sessions {
        if color {
            writeln!(
                writer,
                "  {} 'uhegit resume <repo>' to see full session details.",
                "Tip:".with(Color::DarkGrey)
            )
            .ok();
        } else {
            writeln!(
                writer,
                "  Tip: 'uhegit resume <repo>' to see full session details."
            )
            .ok();
        }
    }
}

fn print_repo_row(
    repo: &Repository,
    name_w: usize,
    branch_w: usize,
    status_w: usize,
    color: bool,
    writer: &mut impl Write,
) {
    let name = truncate(&repo.name, name_w);

    let branch = if let Some(s) = &repo.git_status {
        if s.detached_head {
            "(detached)".to_string()
        } else {
            s.branch.clone().unwrap_or_else(|| "?".to_string())
        }
    } else {
        "?".to_string()
    };
    let branch = truncate(&branch, branch_w);

    let (status_str, status_color) = if let Some(s) = &repo.git_status {
        status_display(
            &s.category,
            s.staged,
            s.modified,
            s.untracked,
            s.conflicted,
            s.deleted,
        )
    } else if let Some(err) = &repo.error {
        (format!("Error: {}", truncate(err, 20)), Color::Red)
    } else {
        ("Unknown".to_string(), Color::DarkGrey)
    };

    let sync_str = if let Some(s) = &repo.git_status {
        sync_display(s.ahead, s.behind, s.upstream.as_deref(), s.sync_stale)
    } else {
        "—".to_string()
    };

    if color {
        let branch_colored = branch.with(Color::Blue);
        let status_colored = status_str.with(status_color);
        writeln!(
            writer,
            "  {:<nw$}  {:<bw$}  {:<sw$}  {}",
            name,
            branch_colored,
            status_colored,
            sync_str,
            nw = name_w,
            bw = branch_w,
            sw = status_w + color_overhead(status_color),
        )
        .ok();
    } else {
        writeln!(
            writer,
            "  {:<nw$}  {:<bw$}  {:<sw$}  {}",
            name,
            branch,
            status_str,
            sync_str,
            nw = name_w,
            bw = branch_w,
            sw = status_w,
        )
        .ok();
    }
}

fn status_display(
    category: &StatusCategory,
    staged: u32,
    modified: u32,
    untracked: u32,
    conflicted: u32,
    deleted: u32,
) -> (String, Color) {
    match category {
        StatusCategory::Conflict => (
            format!(
                "⚡ {} conflict{}",
                conflicted,
                if conflicted == 1 { "" } else { "s" }
            ),
            Color::Red,
        ),
        StatusCategory::Dirty => {
            let mut parts = Vec::new();
            if staged > 0 {
                parts.push(format!("{}↑", staged));
            }
            if modified > 0 {
                parts.push(format!("{}~", modified));
            }
            if deleted > 0 {
                parts.push(format!("{}-", deleted));
            }
            if untracked > 0 {
                parts.push(format!("{}?", untracked));
            }
            let detail = if parts.is_empty() {
                String::new()
            } else {
                format!(" ({})", parts.join(" "))
            };
            (format!("Uncommitted{}", detail), Color::Yellow)
        }
        StatusCategory::Diverged => ("↕ Diverged".to_string(), Color::Magenta),
        StatusCategory::Unpushed => ("↑ Unpushed".to_string(), Color::DarkYellow),
        StatusCategory::Behind => ("↓ Behind upstream".to_string(), Color::Blue),
        StatusCategory::Clean => ("✓ Clean".to_string(), Color::Green),
        StatusCategory::Unknown => ("? Unknown".to_string(), Color::DarkGrey),
    }
}

fn sync_display(
    ahead: Option<u32>,
    behind: Option<u32>,
    upstream: Option<&str>,
    sync_stale: bool,
) -> String {
    if upstream.is_none() {
        return "No upstream".to_string();
    }
    let stale_note = if sync_stale { "*" } else { "" };
    match (ahead, behind) {
        (Some(a), Some(b)) if a > 0 && b > 0 => format!("↑{} ↓{}{}", a, b, stale_note),
        (Some(a), _) if a > 0 => format!("↑ {} ahead{}", a, stale_note),
        (_, Some(b)) if b > 0 => format!("↓ {} behind{}", b, stale_note),
        _ => "Up to date".to_string(),
    }
}

/// Approximate overhead of ANSI escape codes for a given color.
/// Used so that column alignment works correctly when color codes are embedded.
fn color_overhead(_color: Color) -> usize {
    // Crossterm's Stylize adds approximately these bytes for most colors
    // We approximate for common cases; alignment may be off for some terminals
    20
}

pub fn print_resume_output(
    repo: &Repository,
    session: &SessionMemory,
    color: bool,
    writer: &mut impl Write,
) {
    writeln!(writer).ok();
    if color {
        writeln!(
            writer,
            "{}",
            "  UheGit — Resume Session".with(Color::Cyan).bold()
        )
        .ok();
        writeln!(writer, "  {}", "─".repeat(50).with(Color::DarkGrey)).ok();
    } else {
        writeln!(writer, "  UheGit — Resume Session").ok();
        writeln!(writer, "  {}", "─".repeat(50)).ok();
    }
    writeln!(writer).ok();

    let repo_label = if color {
        "Repository:".with(Color::DarkGrey).bold().to_string()
    } else {
        "Repository:".to_string()
    };
    writeln!(writer, "  {} {}", repo_label, repo.name).ok();
    writeln!(
        writer,
        "  {} {}",
        if color {
            "Path:      ".with(Color::DarkGrey).bold().to_string()
        } else {
            "Path:      ".to_string()
        },
        repo.path.display()
    )
    .ok();

    if let Some(s) = &repo.git_status {
        let branch = if s.detached_head {
            "(detached HEAD)".to_string()
        } else {
            s.branch.clone().unwrap_or("?".to_string())
        };
        writeln!(
            writer,
            "  {} {}",
            if color {
                "Branch:    ".with(Color::DarkGrey).bold().to_string()
            } else {
                "Branch:    ".to_string()
            },
            if color {
                branch.with(Color::Blue).to_string()
            } else {
                branch
            }
        )
        .ok();

        if let Some(hash) = &s.commit_hash {
            let subject = s.commit_subject.as_deref().unwrap_or("");
            let rel = s.commit_time_relative.as_deref().unwrap_or("");
            writeln!(
                writer,
                "  {} {} — {} ({})",
                if color {
                    "Commit:    ".with(Color::DarkGrey).bold().to_string()
                } else {
                    "Commit:    ".to_string()
                },
                hash,
                subject,
                rel
            )
            .ok();
        }

        // Working tree summary
        let has_changes =
            s.staged > 0 || s.modified > 0 || s.untracked > 0 || s.deleted > 0 || s.conflicted > 0;
        if has_changes {
            let mut parts = Vec::new();
            if s.conflicted > 0 {
                parts.push(format!("{} conflicted", s.conflicted));
            }
            if s.staged > 0 {
                parts.push(format!("{} staged", s.staged));
            }
            if s.modified > 0 {
                parts.push(format!("{} modified", s.modified));
            }
            if s.deleted > 0 {
                parts.push(format!("{} deleted", s.deleted));
            }
            if s.untracked > 0 {
                parts.push(format!("{} untracked", s.untracked));
            }
            writeln!(
                writer,
                "  {} {}",
                if color {
                    "Changes:   ".with(Color::DarkGrey).bold().to_string()
                } else {
                    "Changes:   ".to_string()
                },
                parts.join(", ")
            )
            .ok();
        } else {
            writeln!(
                writer,
                "  {} Clean working tree",
                if color {
                    "Changes:   ".with(Color::DarkGrey).bold().to_string()
                } else {
                    "Changes:   ".to_string()
                }
            )
            .ok();
        }

        if let Some(up) = &s.upstream {
            let sync = match (s.ahead, s.behind) {
                (Some(a), Some(b)) if a > 0 && b > 0 => {
                    format!("{} ahead, {} behind (diverged) *", a, b)
                }
                (Some(a), _) if a > 0 => format!("{} ahead of {} *", a, up),
                (_, Some(b)) if b > 0 => format!("{} behind {} *", b, up),
                _ => "Up to date *".to_string(),
            };
            writeln!(
                writer,
                "  {} {}",
                if color {
                    "Upstream:  ".with(Color::DarkGrey).bold().to_string()
                } else {
                    "Upstream:  ".to_string()
                },
                sync
            )
            .ok();
            writeln!(
                writer,
                "  {}",
                "  * Based on local tracking refs only. Run 'git fetch' to refresh."
                    .with(Color::DarkGrey)
            )
            .ok();
        }
    }

    writeln!(writer).ok();

    if let Some(note) = &session.note {
        if color {
            writeln!(writer, "  {}", "Last note:".with(Color::DarkGrey).bold()).ok();
        } else {
            writeln!(writer, "  Last note:").ok();
        }
        writeln!(writer, "  {}", note).ok();
        writeln!(writer).ok();
    }

    if let Some(task) = &session.next_task {
        if color {
            writeln!(writer, "  {}", "Next task:".with(Color::DarkGrey).bold()).ok();
        } else {
            writeln!(writer, "  Next task:").ok();
        }
        writeln!(
            writer,
            "  {}",
            if color {
                task.as_str().with(Color::Cyan).to_string()
            } else {
                task.clone()
            }
        )
        .ok();
        writeln!(writer).ok();
    }

    if let Some(ts) = &session.checkpoint_time {
        if color {
            writeln!(
                writer,
                "  {}",
                "Last checkpoint:".with(Color::DarkGrey).bold()
            )
            .ok();
        } else {
            writeln!(writer, "  Last checkpoint:").ok();
        }
        writeln!(
            writer,
            "  {} at {}",
            session.checkpoint_hash.as_deref().unwrap_or("?"),
            ts.format("%Y-%m-%d %H:%M UTC")
        )
        .ok();
        if let Some(summary) = &session.checkpoint_summary {
            writeln!(writer, "  {}", summary).ok();
        }
        writeln!(writer).ok();
    }

    // Continue command
    if color {
        writeln!(writer, "  {}", "Continue:".with(Color::DarkGrey).bold()).ok();
    } else {
        writeln!(writer, "  Continue:").ok();
    }
    let path_str = repo.path.display();
    writeln!(writer, "  cd \"{}\"", path_str).ok();
    writeln!(writer).ok();
}

pub fn print_scan_result(
    found: usize,
    newly_added: usize,
    already_known: usize,
    skipped: usize,
    inaccessible: usize,
    color: bool,
    writer: &mut impl Write,
) {
    writeln!(writer).ok();
    if color {
        writeln!(writer, "{}", "  Scan complete".with(Color::Cyan).bold()).ok();
    } else {
        writeln!(writer, "  Scan complete").ok();
    }
    writeln!(writer, "  Discovered:   {}", found).ok();
    writeln!(writer, "  New:          {}", newly_added).ok();
    writeln!(writer, "  Already known: {}", already_known).ok();
    writeln!(writer, "  Skipped dirs: {}", skipped).ok();
    if inaccessible > 0 {
        writeln!(writer, "  Inaccessible: {}", inaccessible).ok();
    }
    writeln!(writer).ok();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 1 {
        format!("{}…", &s[..max - 1])
    } else {
        s[..max].to_string()
    }
}
