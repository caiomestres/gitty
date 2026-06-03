use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::Result;
use crate::repository::Repository;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub commit_hash: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub branch: String,
    pub repo_id: Uuid,
    pub repo_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeWindow {
    Day,
    Week,
    Month,
}

impl TimeWindow {
    pub fn cutoff(&self) -> OffsetDateTime {
        let now = OffsetDateTime::now_utc();
        match self {
            TimeWindow::Day => now - time::Duration::days(1),
            TimeWindow::Week => now - time::Duration::days(7),
            TimeWindow::Month => now - time::Duration::days(30),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grouping {
    Author,
    Repository,
    Branch,
}

// ---------------------------------------------------------------------------
// Scanning
// ---------------------------------------------------------------------------

/// Scan recent commits from each repo, filtering by time window.
/// `all_branches` contains repo IDs that should scan all local branches
/// (not just HEAD).
pub fn scan_changes(
    repos: &[&Repository],
    window: TimeWindow,
    all_branches: &HashSet<Uuid>,
) -> Result<Vec<ChangeEntry>> {
    let cutoff = window.cutoff();
    let mut entries = Vec::new();

    for repo in repos {
        if repo.state == crate::repository::RepositoryState::Missing {
            continue;
        }

        let git_repo = match git2::Repository::open(&repo.path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if git_repo.is_empty().unwrap_or(true) {
            continue;
        }

        let branches_to_scan = if all_branches.contains(&repo.id) {
            collect_local_branches(&git_repo)
        } else {
            head_branch(&git_repo)
        };

        let mut seen = HashSet::new();
        let name = repo.display_name().to_string();

        for (branch_name, oid) in &branches_to_scan {
            let mut walk = match git_repo.revwalk() {
                Ok(w) => w,
                Err(_) => continue,
            };
            walk.set_sorting(git2::Sort::TIME).ok();
            if walk.push(*oid).is_err() {
                continue;
            }

            for maybe_oid in walk {
                let commit_oid = match maybe_oid {
                    Ok(o) => o,
                    Err(_) => break,
                };

                if !seen.insert(commit_oid) {
                    continue;
                }

                let commit = match git_repo.find_commit(commit_oid) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let author_time = commit.author().when();
                let ts = match OffsetDateTime::from_unix_timestamp(author_time.seconds()) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if ts < cutoff {
                    break;
                }

                let offset = time::UtcOffset::from_whole_seconds(author_time.offset_minutes() * 60)
                    .unwrap_or(time::UtcOffset::UTC);
                let date_str = ts
                    .to_offset(offset)
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| author_time.seconds().to_string());

                entries.push(ChangeEntry {
                    commit_hash: commit_oid.to_string(),
                    author: commit.author().name().unwrap_or_default().to_string(),
                    date: date_str,
                    subject: commit.summary().ok().flatten().unwrap_or("").to_string(),
                    branch: branch_name.clone(),
                    repo_id: repo.id,
                    repo_name: name.clone(),
                });
            }
        }
    }

    entries.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(entries)
}

fn head_branch(repo: &git2::Repository) -> Vec<(String, git2::Oid)> {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return vec![],
    };
    let name = head.shorthand().unwrap_or("HEAD").to_string();
    match head.target() {
        Some(oid) => vec![(name, oid)],
        None => vec![],
    }
}

fn collect_local_branches(repo: &git2::Repository) -> Vec<(String, git2::Oid)> {
    let mut result = Vec::new();
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for (branch, _) in branches.flatten() {
            if let (Ok(Some(name)), Some(oid)) = (branch.name(), branch.get().target()) {
                result.push((name.to_string(), oid));
            }
        }
    }
    if result.is_empty() {
        return head_branch(repo);
    }
    result
}

// ---------------------------------------------------------------------------
// Grouping
// ---------------------------------------------------------------------------

/// Group change entries by the selected dimension.
pub fn group_changes(entries: &[ChangeEntry], by: Grouping) -> BTreeMap<String, Vec<&ChangeEntry>> {
    let mut map: BTreeMap<String, Vec<&ChangeEntry>> = BTreeMap::new();
    for entry in entries {
        let key = match by {
            Grouping::Author => entry.author.clone(),
            Grouping::Repository => entry.repo_name.clone(),
            Grouping::Branch => entry.branch.clone(),
        };
        map.entry(key).or_default().push(entry);
    }
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn init_repo_with_commits(dir: &Path, count: usize) {
        let repo = git2::Repository::init(dir).unwrap();
        let workdir = repo.workdir().unwrap();
        let sig = git2::Signature::now("Alice", "alice@example.com").unwrap();

        for i in 0..count {
            let filename = format!("file{i}.txt");
            std::fs::write(workdir.join(&filename), format!("content {i}")).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(Path::new(&filename)).unwrap();
            index.write().unwrap();
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();

            if i == 0 {
                repo.commit(Some("HEAD"), &sig, &sig, &format!("commit {i}"), &tree, &[])
                    .unwrap();
            } else {
                let parent = repo.head().unwrap().peel_to_commit().unwrap();
                repo.commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    &format!("commit {i}"),
                    &tree,
                    &[&parent],
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn scan_changes_finds_recent_commits() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commits(dir.path(), 3);

        let repo = Repository::new(dir.path().to_path_buf(), Some("fp".into()));
        let entries = scan_changes(&[&repo], TimeWindow::Week, &HashSet::new()).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn scan_changes_empty_repo() {
        let dir = tempfile::tempdir().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        let repo = Repository::new(dir.path().to_path_buf(), None);
        let entries = scan_changes(&[&repo], TimeWindow::Week, &HashSet::new()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn scan_changes_skips_missing_repo() {
        let mut repo = Repository::new("/nonexistent/path".into(), None);
        repo.state = crate::repository::RepositoryState::Missing;
        let entries = scan_changes(&[&repo], TimeWindow::Week, &HashSet::new()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn group_changes_by_author() {
        let entries = vec![
            ChangeEntry {
                commit_hash: "a".into(),
                author: "Alice".into(),
                date: "2024-01-01T00:00:00Z".into(),
                subject: "fix".into(),
                branch: "main".into(),
                repo_id: Uuid::new_v4(),
                repo_name: "repo1".into(),
            },
            ChangeEntry {
                commit_hash: "b".into(),
                author: "Bob".into(),
                date: "2024-01-01T00:00:00Z".into(),
                subject: "feat".into(),
                branch: "main".into(),
                repo_id: Uuid::new_v4(),
                repo_name: "repo1".into(),
            },
            ChangeEntry {
                commit_hash: "c".into(),
                author: "Alice".into(),
                date: "2024-01-01T00:00:00Z".into(),
                subject: "docs".into(),
                branch: "main".into(),
                repo_id: Uuid::new_v4(),
                repo_name: "repo2".into(),
            },
        ];

        let grouped = group_changes(&entries, Grouping::Author);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped["Alice"].len(), 2);
        assert_eq!(grouped["Bob"].len(), 1);
    }

    #[test]
    fn group_changes_by_repository() {
        let id = Uuid::new_v4();
        let entries = vec![
            ChangeEntry {
                commit_hash: "a".into(),
                author: "Alice".into(),
                date: "2024-01-01T00:00:00Z".into(),
                subject: "fix".into(),
                branch: "main".into(),
                repo_id: id,
                repo_name: "repo1".into(),
            },
            ChangeEntry {
                commit_hash: "b".into(),
                author: "Bob".into(),
                date: "2024-01-01T00:00:00Z".into(),
                subject: "feat".into(),
                branch: "dev".into(),
                repo_id: id,
                repo_name: "repo1".into(),
            },
        ];

        let grouped = group_changes(&entries, Grouping::Repository);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped["repo1"].len(), 2);
    }

    #[test]
    fn group_changes_empty_input() {
        let entries: Vec<ChangeEntry> = vec![];
        let grouped = group_changes(&entries, Grouping::Branch);
        assert!(grouped.is_empty());
    }

    #[test]
    fn time_window_cutoff_is_in_past() {
        let now = OffsetDateTime::now_utc();
        assert!(TimeWindow::Day.cutoff() < now);
        assert!(TimeWindow::Week.cutoff() < now);
        assert!(TimeWindow::Month.cutoff() < now);
    }

    #[test]
    fn time_window_serializes_lowercase() {
        let json = serde_json::to_string(&TimeWindow::Week).unwrap();
        assert_eq!(json, "\"week\"");
    }

    #[test]
    fn scan_changes_populates_fields() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commits(dir.path(), 1);

        let repo = Repository::new(dir.path().to_path_buf(), Some("fp".into()));
        let entries = scan_changes(&[&repo], TimeWindow::Week, &HashSet::new()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].author, "Alice");
        assert!(!entries[0].commit_hash.is_empty());
        assert_eq!(entries[0].subject, "commit 0");
    }
}
