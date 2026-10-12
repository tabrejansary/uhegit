use crate::types::{GitStatus, StatusCategory};

/// Parse `git status --porcelain=v1 -z --branch` output into a GitStatus.
///
/// Uses the stable, machine-readable porcelain v1 format with NUL separators.
/// This handles unusual filenames (spaces, tabs, Unicode, newlines) correctly.
pub fn parse_porcelain_v1(output: &[u8]) -> GitStatus {
    let text = String::from_utf8_lossy(output);

    let mut staged = 0u32;
    let mut modified = 0u32;
    let mut untracked = 0u32;
    let mut conflicted = 0u32;
    let mut deleted = 0u32;

    let mut branch: Option<String> = None;
    let mut detached_head = false;
    let mut upstream: Option<String> = None;
    let mut ahead: Option<u32> = None;
    let mut behind: Option<u32> = None;

    // Split by NUL (each entry is terminated by NUL in -z mode)
    // The branch header line is separated by newline from the status entries
    let mut lines = text.split('\0');

    // First entry contains the branch header (## line) and possibly the first file entry
    if let Some(first) = lines.next() {
        for line in first.lines() {
            if let Some(rest) = line.strip_prefix("## ") {
                parse_branch_header(
                    rest,
                    &mut branch,
                    &mut detached_head,
                    &mut upstream,
                    &mut ahead,
                    &mut behind,
                );
            } else if line.len() >= 2 {
                count_status_entry(
                    line,
                    &mut staged,
                    &mut modified,
                    &mut untracked,
                    &mut conflicted,
                    &mut deleted,
                );
            }
        }
    }

    // Remaining NUL-separated entries (for renamed files, the next entry is the original name)
    let mut skip_next = false;
    for entry in lines {
        if skip_next {
            skip_next = false;
            continue;
        }
        if entry.len() < 2 {
            continue;
        }
        // Check if this is a rename (R or C in xy codes) — next NUL chunk is the old name
        let xy = &entry[..2];
        if xy == "R " || xy == " R" || xy == "RR" || xy == "C " || xy == " C" {
            skip_next = true;
        }
        count_status_entry(
            entry,
            &mut staged,
            &mut modified,
            &mut untracked,
            &mut conflicted,
            &mut deleted,
        );
    }

    let category = compute_category(
        staged, modified, untracked, conflicted, deleted, ahead, behind,
    );

    GitStatus {
        branch,
        detached_head,
        commit_hash: None,    // filled separately
        commit_subject: None, // filled separately
        commit_time_relative: None,
        staged,
        modified,
        untracked,
        conflicted,
        deleted,
        upstream,
        ahead,
        behind,
        category,
        sync_stale: true, // always stale until explicit fetch
    }
}

/// Parse the branch header line from porcelain v1 output.
///
/// Format: `## <branch>...<upstream> [ahead N] [behind N]`
/// Detached HEAD: `## HEAD (no branch)`
fn parse_branch_header(
    rest: &str,
    branch: &mut Option<String>,
    detached_head: &mut bool,
    upstream: &mut Option<String>,
    ahead: &mut Option<u32>,
    behind: &mut Option<u32>,
) {
    if rest.starts_with("HEAD (no branch)") || rest.starts_with("No commits yet on") {
        *detached_head = true;
        return;
    }

    // Split on "..." to find upstream
    if let Some(dot_pos) = rest.find("...") {
        let local = &rest[..dot_pos];
        let remainder = &rest[dot_pos + 3..];

        *branch = Some(local.to_string());

        // remainder is like "origin/main [ahead 2, behind 1]"
        if let Some(bracket_pos) = remainder.find(" [") {
            let up = &remainder[..bracket_pos];
            if !up.is_empty() {
                *upstream = Some(up.to_string());
            }
            let bracket_content = &remainder[bracket_pos + 2..];
            let bracket_content = bracket_content.trim_end_matches(']');
            parse_ahead_behind(bracket_content, ahead, behind);
        } else {
            if !remainder.is_empty() {
                *upstream = Some(remainder.to_string());
            }
        }
    } else {
        // No upstream — local-only branch or initial commit
        let b = rest.trim();
        if !b.is_empty() {
            *branch = Some(b.to_string());
        }
    }
}

fn parse_ahead_behind(s: &str, ahead: &mut Option<u32>, behind: &mut Option<u32>) {
    for part in s.split(',') {
        let part = part.trim();
        if let Some(n) = part.strip_prefix("ahead ") {
            *ahead = n.trim().parse().ok();
        } else if let Some(n) = part.strip_prefix("behind ") {
            *behind = n.trim().parse().ok();
        }
    }
}

/// Count a single status entry's XY codes.
///
/// XY codes from `git status --porcelain=v1`:
/// X = index (staged) status, Y = working tree status
/// ' ' = unmodified, M = modified, A = added, D = deleted,
/// R = renamed, C = copied, U = updated but unmerged, ? = untracked, ! = ignored
fn count_status_entry(
    entry: &str,
    staged: &mut u32,
    modified: &mut u32,
    untracked: &mut u32,
    conflicted: &mut u32,
    deleted: &mut u32,
) {
    if entry.len() < 3 {
        return;
    }
    let x = entry.chars().next().unwrap_or(' ');
    let y = entry.chars().nth(1).unwrap_or(' ');

    match (x, y) {
        ('?', '?') => *untracked += 1,
        ('!', '!') => {} // ignored — skip
        // Conflict states
        ('U', _) | (_, 'U') | ('A', 'A') | ('D', 'D') => *conflicted += 1,
        _ => {
            // Index changes
            if x != ' ' && x != '?' {
                *staged += 1;
            }
            // Working tree changes
            match y {
                'M' => *modified += 1,
                'D' => *deleted += 1,
                _ => {}
            }
        }
    }
}

fn compute_category(
    staged: u32,
    modified: u32,
    untracked: u32,
    conflicted: u32,
    deleted: u32,
    ahead: Option<u32>,
    behind: Option<u32>,
) -> StatusCategory {
    if conflicted > 0 {
        return StatusCategory::Conflict;
    }
    let has_changes = staged > 0 || modified > 0 || untracked > 0 || deleted > 0;
    if has_changes {
        return StatusCategory::Dirty;
    }
    let a = ahead.unwrap_or(0);
    let b = behind.unwrap_or(0);
    if a > 0 && b > 0 {
        return StatusCategory::Diverged;
    }
    if a > 0 {
        return StatusCategory::Unpushed;
    }
    if b > 0 {
        return StatusCategory::Behind;
    }
    StatusCategory::Clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_no_upstream() {
        let input = b"## main\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.branch, Some("main".to_string()));
        assert_eq!(status.staged, 0);
        assert_eq!(status.category, StatusCategory::Clean);
    }

    #[test]
    fn test_parse_clean_with_upstream() {
        let input = b"## main...origin/main\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.branch, Some("main".to_string()));
        assert_eq!(status.upstream, Some("origin/main".to_string()));
        assert_eq!(status.ahead, None);
        assert_eq!(status.behind, None);
        assert_eq!(status.category, StatusCategory::Clean);
    }

    #[test]
    fn test_parse_ahead() {
        let input = b"## feat/x...origin/feat/x [ahead 3]\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.ahead, Some(3));
        assert_eq!(status.behind, None);
        assert_eq!(status.category, StatusCategory::Unpushed);
    }

    #[test]
    fn test_parse_behind() {
        let input = b"## main...origin/main [behind 2]\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.behind, Some(2));
        assert_eq!(status.category, StatusCategory::Behind);
    }

    #[test]
    fn test_parse_diverged() {
        let input = b"## main...origin/main [ahead 1, behind 2]\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.ahead, Some(1));
        assert_eq!(status.behind, Some(2));
        assert_eq!(status.category, StatusCategory::Diverged);
    }

    #[test]
    fn test_parse_modified() {
        let input = b"## main\0 M src/main.rs\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.modified, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_parse_staged() {
        let input = b"## main\0M  src/main.rs\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.staged, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_parse_untracked() {
        let input = b"## main\0?? new_file.txt\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.untracked, 1);
        assert_eq!(status.category, StatusCategory::Dirty);
    }

    #[test]
    fn test_parse_conflicted() {
        let input = b"## main\0UU conflicted.txt\0";
        let status = parse_porcelain_v1(input);
        assert_eq!(status.conflicted, 1);
        assert_eq!(status.category, StatusCategory::Conflict);
    }

    #[test]
    fn test_parse_detached_head() {
        let input = b"## HEAD (no branch)\0";
        let status = parse_porcelain_v1(input);
        assert!(status.detached_head);
        assert_eq!(status.branch, None);
    }
}
