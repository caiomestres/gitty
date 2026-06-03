use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::group::Group;
use crate::health::HealthThresholds;
use crate::macro_def::MacroDef;

/// Lifecycle state of a registered Repository.
///
/// `Missing` means the recorded path no longer exists on disk; the Repository
/// is retained (never silently deleted) so identity and organization survive
/// relocation (see CONTEXT.md and ADR-0005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryState {
    #[default]
    Active,
    Missing,
}

/// A local Git repository tracked by Gitty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub path: PathBuf,
    /// Content fingerprint (root-commit OID hex). `None` when the repository
    /// has no commits. Used for collision-safe re-linking (ADR-0005).
    #[serde(default)]
    pub fingerprint: Option<String>,
    #[serde(default)]
    pub state: RepositoryState,
    // Pre-modelled for Milestone 4 (Groups & Tags). Defaulted and unused in slice 1.
    #[serde(default)]
    pub group_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Repository {
    pub fn display_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<repo>")
    }

    pub fn new(path: PathBuf, fingerprint: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            fingerprint,
            state: RepositoryState::Active,
            group_id: None,
            tags: Vec::new(),
        }
    }
}

/// A filesystem directory scanned recursively for `.git` directories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanRoot {
    pub path: PathBuf,
}

/// The single implicit default Workspace for v1 (CONTEXT.md): its Scan Roots
/// and the Repository registry discovered beneath them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub scan_roots: Vec<ScanRoot>,
    #[serde(default)]
    pub repositories: Vec<Repository>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub macros: Vec<MacroDef>,
    #[serde(default)]
    pub health_thresholds: HealthThresholds,
}

impl Workspace {
    /// Register a Scan Root. Returns `false` if it was already present.
    pub fn add_scan_root(&mut self, path: PathBuf) -> bool {
        if self.scan_roots.iter().any(|sr| sr.path == path) {
            return false;
        }
        self.scan_roots.push(ScanRoot { path });
        true
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.path == path)
    }

    pub fn find_by_id(&self, id: Uuid) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.id == id)
    }

    pub(crate) fn find_repo_mut(&mut self, id: Uuid) -> Option<&mut Repository> {
        self.repositories.iter_mut().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_scan_root_dedupes() {
        let mut ws = Workspace::default();
        assert!(ws.add_scan_root(PathBuf::from("/a")));
        assert!(!ws.add_scan_root(PathBuf::from("/a")));
        assert_eq!(ws.scan_roots.len(), 1);
    }

    #[test]
    fn find_by_path_and_id() {
        let mut ws = Workspace::default();
        let repo = Repository::new(PathBuf::from("/a/b"), Some("abc".into()));
        let id = repo.id;
        ws.repositories.push(repo);
        assert!(ws.find_by_path(Path::new("/a/b")).is_some());
        assert!(ws.find_by_id(id).is_some());
        assert!(ws.find_by_path(Path::new("/nope")).is_none());
    }

    #[test]
    fn repository_state_serializes_lowercase() {
        let json = serde_json::to_string(&RepositoryState::Missing).unwrap();
        assert_eq!(json, "\"missing\"");
    }
}
