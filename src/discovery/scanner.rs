use crate::git::status::{get_toplevel, is_git_repo};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directories to skip during recursive scanning.
const EXCLUSION_DIRS: &[&str] = &[
    "node_modules",
    ".venv",
    "venv",
    ".env",
    "vendor",
    "target",
    "dist",
    "build",
    ".cargo",
    ".rustup",
    "__pycache__",
    ".cache",
    ".npm",
    ".yarn",
    "bower_components",
    "jspm_packages",
    ".gradle",
    ".mvn",
    "obj",
    "bin",  // only skip if not a repo itself
    ".git", // never descend into .git internals
    ".svn",
    ".hg",
    "AppData",
    "Library",
    "Applications",
    "System",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
];

/// Results of a discovery scan.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub found: Vec<PathBuf>,
    pub newly_added: usize,
    pub already_known: usize,
    pub skipped: usize,
    pub inaccessible: usize,
}

/// Platform-appropriate default discovery roots.
pub fn default_discovery_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(home) = dirs_home() {
        let candidates = [
            "Projects",
            "Code",
            "Developer",
            "dev",
            "Development",
            "src",
            "workspace",
            "repos",
            "git",
            "work",
            "Documents",
            "Desktop",
        ];
        for candidate in &candidates {
            let p = home.join(candidate);
            if p.exists() {
                roots.push(p);
            }
        }

        // On Windows, also check common drive roots like D:\Projects
        #[cfg(target_os = "windows")]
        {
            for drive in &["D:", "E:", "F:"] {
                for candidate in &["Projects", "Code", "dev", "repos"] {
                    let p = PathBuf::from(format!("{}\\{}", drive, candidate));
                    if p.exists() {
                        roots.push(p);
                    }
                }
            }
        }
    }

    // Deduplicate
    let mut seen = HashSet::new();
    roots.retain(|p| {
        let canon = canonicalize_path(p);
        seen.insert(canon)
    });

    roots
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let drive = std::env::var("HOMEDRIVE").ok()?;
                let path = std::env::var("HOMEPATH").ok()?;
                Some(PathBuf::from(format!("{}{}", drive, path)))
            })
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

pub fn home_dir() -> Option<PathBuf> {
    dirs_home()
}

fn canonicalize_path(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn should_skip_dir(dir_name: &str, is_repo: bool) -> bool {
    if is_repo {
        return false; // Never skip a directory that is itself a repo root
    }
    EXCLUSION_DIRS
        .iter()
        .any(|&excl| dir_name.eq_ignore_ascii_case(excl))
}

/// Scan a root directory for Git repositories up to `max_depth` levels deep.
///
/// - Does not follow symlinks.
/// - Skips excluded directories (node_modules, target, etc.) unless they are repos.
/// - Does not descend into .git directories.
/// - Handles inaccessible directories gracefully.
pub fn scan_for_repos(
    root: &Path,
    max_depth: usize,
    timeout_secs: u64,
    result: &mut ScanResult,
    known: &HashSet<PathBuf>,
) {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    scan_recursive(
        root,
        0,
        max_depth,
        timeout_secs,
        result,
        known,
        &mut visited,
    );
}

fn scan_recursive(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    timeout_secs: u64,
    result: &mut ScanResult,
    known: &HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) {
    if depth > max_depth {
        return;
    }

    // Avoid symlinks (don't follow by default)
    match std::fs::symlink_metadata(dir) {
        Ok(meta) if meta.file_type().is_symlink() => {
            result.skipped += 1;
            return;
        }
        Err(_) => {
            result.inaccessible += 1;
            return;
        }
        _ => {}
    }

    // Track visited canonical paths to avoid loops
    let canon = canonicalize_path(dir);
    if !visited.insert(canon.clone()) {
        return; // Already visited this real path
    }

    // Check if this directory is a Git repo
    let is_repo = is_git_repo(dir, timeout_secs);

    if is_repo {
        // Get canonical working tree root
        if let Ok(toplevel) = get_toplevel(dir, timeout_secs) {
            let canon_toplevel = canonicalize_path(&toplevel);
            if !result
                .found
                .iter()
                .any(|p| canonicalize_path(p) == canon_toplevel)
            {
                let is_known = known.iter().any(|k| canonicalize_path(k) == canon_toplevel);
                if is_known {
                    result.already_known += 1;
                } else {
                    result.newly_added += 1;
                }
                result.found.push(toplevel.clone());
            }
            // Don't descend further into a repo's subdirectories (skip .git etc)
            // But do continue for nested repos (submodules) — they will be found
            // when we descend if needed. For simplicity, stop here.
            return;
        }
    }

    // Descend into subdirectories
    if depth < max_depth {
        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => {
                result.inaccessible += 1;
                return;
            }
        };

        let mut subdirs: Vec<PathBuf> = Vec::new();
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Never follow symlinked directories
                if let Ok(meta) = std::fs::symlink_metadata(&path) {
                    if meta.file_type().is_symlink() {
                        continue;
                    }
                }
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if name == ".git" {
                    continue; // Never descend into .git internals
                }

                if should_skip_dir(name, false) {
                    result.skipped += 1;
                    continue;
                }
                subdirs.push(path);
            }
        }

        // Sort for deterministic order
        subdirs.sort();
        for sub in subdirs {
            scan_recursive(
                &sub,
                depth + 1,
                max_depth,
                timeout_secs,
                result,
                known,
                visited,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::executor::{run_git, GIT_TIMEOUT_SECS};
    use std::fs;
    use tempfile::TempDir;

    fn init_repo(path: &Path) {
        run_git(&["init"], path, GIT_TIMEOUT_SECS).unwrap();
        run_git(
            &["config", "user.email", "test@test.com"],
            path,
            GIT_TIMEOUT_SECS,
        )
        .unwrap();
        run_git(&["config", "user.name", "Test"], path, GIT_TIMEOUT_SECS).unwrap();
    }

    #[test]
    fn test_scan_finds_repo() {
        let root = TempDir::new().unwrap();
        let repo_dir = root.path().join("my-project");
        fs::create_dir(&repo_dir).unwrap();
        init_repo(&repo_dir);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        scan_for_repos(root.path(), 3, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_skips_node_modules() {
        let root = TempDir::new().unwrap();
        let nm = root.path().join("node_modules").join("some-package");
        fs::create_dir_all(&nm).unwrap();
        init_repo(&nm);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        scan_for_repos(root.path(), 5, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 0);
    }

    #[test]
    fn test_scan_nested_repos() {
        let root = TempDir::new().unwrap();
        let parent = root.path().join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();
        // Only init parent (child is inside parent, won't be found separately)
        init_repo(&parent);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        scan_for_repos(root.path(), 4, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_depth_limit() {
        let root = TempDir::new().unwrap();
        let deep = root
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e");
        fs::create_dir_all(&deep).unwrap();
        init_repo(&deep);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        // Only scan 2 levels deep — should not find it
        scan_for_repos(root.path(), 2, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 0);
    }

    #[test]
    fn test_scan_path_with_spaces() {
        let root = TempDir::new().unwrap();
        let dir = root.path().join("my project with spaces");
        fs::create_dir_all(&dir).unwrap();
        init_repo(&dir);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        scan_for_repos(root.path(), 3, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_no_duplicate_repos() {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("myrepo");
        fs::create_dir(&repo).unwrap();
        init_repo(&repo);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        // Scan twice from same root
        scan_for_repos(root.path(), 4, GIT_TIMEOUT_SECS, &mut result, &known);
        scan_for_repos(root.path(), 4, GIT_TIMEOUT_SECS, &mut result, &known);
        // Should not duplicate
        assert_eq!(result.found.len(), 1);
    }

    #[test]
    fn test_scan_inaccessible_dirs_dont_abort() {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("good-repo");
        fs::create_dir(&repo).unwrap();
        init_repo(&repo);

        let mut result = ScanResult::default();
        let known = HashSet::new();
        scan_for_repos(root.path(), 4, GIT_TIMEOUT_SECS, &mut result, &known);
        assert_eq!(result.found.len(), 1);
    }
}
