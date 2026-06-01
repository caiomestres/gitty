//! Read-only Git inspection via `git2` (libgit2).
//!
//! All network and write operations shell out to the `git` CLI (ADR-0001), so
//! this module never touches the network — it only reads local repository state.

use std::path::Path;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

use crate::error::Result;

/// A snapshot of a Repository's current Git state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Current branch shorthand. `None` when detached or on an unborn branch.
    pub branch: Option<String>,
    pub detached: bool,
    /// Whether the working tree or index has any changes (untracked included).
    pub dirty: bool,
    /// Tracking-branch divergence. `None` when there is no upstream.
    pub upstream: Option<Upstream>,
    /// The HEAD commit summary. `None` when the repository has no commits.
    pub head: Option<CommitSummary>,
    pub changed_files: Vec<ChangedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upstream {
    pub name: String,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub short_id: String,
    pub author: String,
    /// RFC3339 timestamp of the commit (author time).
    pub date: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: String,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

/// Compute the content fingerprint used for re-linking (ADR-0005): the OID of
/// the repository's root commit. Returns `None` for an empty repository.
pub fn root_fingerprint(path: &Path) -> Result<Option<String>> {
    let repo = git2::Repository::open(path)?;
    if repo.is_empty()? {
        return Ok(None);
    }
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;
    match walk.next() {
        Some(Ok(oid)) => Ok(Some(oid.to_string())),
        _ => Ok(None),
    }
}

/// Read the full status snapshot for the repository at `path`.
pub fn read_status(path: &Path) -> Result<RepositoryStatus> {
    let repo = git2::Repository::open(path)?;

    let detached = repo.head_detached().unwrap_or(false);

    let branch = if detached {
        None
    } else {
        repo.head()
            .ok()
            .and_then(|h| h.shorthand().ok().map(str::to_string))
    };

    let changed_files = collect_changed_files(&repo)?;
    let dirty = !changed_files.is_empty();
    let upstream = compute_upstream(&repo).unwrap_or(None);
    let head = head_summary(&repo)?;

    Ok(RepositoryStatus {
        branch,
        detached,
        dirty,
        upstream,
        head,
        changed_files,
    })
}

fn collect_changed_files(repo: &git2::Repository) -> Result<Vec<ChangedFile>> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut files = Vec::new();
    for entry in statuses.iter() {
        if let Some(status) = classify(entry.status()) {
            files.push(ChangedFile {
                path: entry.path().unwrap_or_default().to_string(),
                status,
            });
        }
    }
    Ok(files)
}

fn classify(s: git2::Status) -> Option<ChangeStatus> {
    use git2::Status as St;
    if s.intersects(St::INDEX_RENAMED | St::WT_RENAMED) {
        return Some(ChangeStatus::Renamed);
    }
    if s.intersects(St::INDEX_DELETED | St::WT_DELETED) {
        return Some(ChangeStatus::Deleted);
    }
    if s.contains(St::INDEX_NEW) {
        return Some(ChangeStatus::Added);
    }
    if s.contains(St::WT_NEW) {
        return Some(ChangeStatus::Untracked);
    }
    if s.intersects(St::INDEX_MODIFIED | St::WT_MODIFIED | St::INDEX_TYPECHANGE | St::WT_TYPECHANGE)
    {
        return Some(ChangeStatus::Modified);
    }
    None
}

fn compute_upstream(repo: &git2::Repository) -> Result<Option<Upstream>> {
    if repo.head_detached().unwrap_or(false) {
        return Ok(None);
    }
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };
    if !head.is_branch() {
        return Ok(None);
    }
    let local_oid = match head.target() {
        Some(oid) => oid,
        None => return Ok(None),
    };
    let shorthand = match head.shorthand() {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(None),
    };

    let branch = repo.find_branch(&shorthand, git2::BranchType::Local)?;
    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => return Ok(None), // no tracking branch configured
    };
    let up_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return Ok(None),
    };
    let name = upstream.name()?.unwrap_or_default().to_string();
    let (ahead, behind) = repo.graph_ahead_behind(local_oid, up_oid)?;

    Ok(Some(Upstream {
        name,
        ahead,
        behind,
    }))
}

fn head_summary(repo: &git2::Repository) -> Result<Option<CommitSummary>> {
    if repo.is_empty()? {
        return Ok(None);
    }
    let oid = match repo.head().ok().and_then(|h| h.target()) {
        Some(oid) => oid,
        None => return Ok(None),
    };
    let commit = repo.find_commit(oid)?;
    let short_id = commit
        .as_object()
        .short_id()?
        .as_str()
        .unwrap_or_default()
        .to_string();
    let author = commit.author();

    Ok(Some(CommitSummary {
        short_id,
        author: author.name().unwrap_or_default().to_string(),
        date: format_git_time(author.when()),
        subject: commit
            .summary()
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
    }))
}

/// Format a libgit2 timestamp as RFC3339, falling back to the raw epoch on the
/// (unexpected) chance the date is out of range.
fn format_git_time(t: git2::Time) -> String {
    let offset = UtcOffset::from_whole_seconds(t.offset_minutes() * 60).unwrap_or(UtcOffset::UTC);
    match OffsetDateTime::from_unix_timestamp(t.seconds()) {
        Ok(dt) => dt
            .to_offset(offset)
            .format(&Rfc3339)
            .unwrap_or_else(|_| t.seconds().to_string()),
        Err(_) => t.seconds().to_string(),
    }
}
