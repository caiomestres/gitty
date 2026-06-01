//! Scan Root discovery: walk a directory tree and find Git repositories.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{CoreError, Result};
use crate::git::read;

/// Directory names pruned from the walk to keep scans fast (D8). These are
/// common heavy build/dependency directories that never contain repositories
/// the user cares about. Overridable via Config in a later slice.
pub const DEFAULT_IGNORE_DIRS: &[&str] =
    &["node_modules", "target", ".venv", "dist", "build", ".next"];

/// A repository found on disk during a scan, before it is reconciled into the
/// registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredRepo {
    pub path: PathBuf,
    pub fingerprint: Option<String>,
}

/// Recursively discover Git repositories under `root`.
///
/// Behavior (locked in D8): descends into nested repositories, never descends
/// into a `.git` directory or an ignored directory, does not follow symlinks,
/// and only detects standard (non-bare) repositories — those with a `.git`
/// child. Paths are canonicalized and de-duplicated.
pub fn scan(root: &Path) -> Result<Vec<DiscoveredRepo>> {
    if !root.exists() {
        return Err(CoreError::PathNotFound(root.to_path_buf()));
    }

    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut repo_paths: Vec<PathBuf> = Vec::new();

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(is_not_pruned);

    for entry in walker {
        // Skip entries we cannot read (permission denied, etc.) and continue.
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        // A standard repository is a directory containing a `.git` child
        // (a directory normally, or a file for submodules/worktrees).
        if entry.path().join(".git").exists() {
            let canonical =
                dunce::canonicalize(entry.path()).unwrap_or_else(|_| entry.path().to_path_buf());
            if seen.insert(canonical.clone()) {
                repo_paths.push(canonical);
            }
        }
    }

    let mut repos = Vec::with_capacity(repo_paths.len());
    for path in repo_paths {
        let fingerprint = read::root_fingerprint(&path).unwrap_or(None);
        repos.push(DiscoveredRepo { path, fingerprint });
    }
    Ok(repos)
}

fn is_not_pruned(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }
    match entry.file_name().to_str() {
        Some(name) => name != ".git" && !DEFAULT_IGNORE_DIRS.contains(&name),
        None => true,
    }
}
