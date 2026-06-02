//! Merge freshly-discovered repositories into the Repository registry,
//! handling idempotent rescans, the `Missing` state, and collision-safe
//! re-linking (ADR-0005).

use std::collections::HashMap;

use crate::repository::{Repository, RepositoryState, Workspace};
use crate::scan::DiscoveredRepo;

/// Summary of what a reconcile pass changed, for user-facing reporting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcileReport {
    /// Repositories discovered on disk this pass.
    pub found: usize,
    /// Newly registered (not matched to anything existing).
    pub new: usize,
    /// Re-linked to an existing registry entry that had moved.
    pub relinked: usize,
    /// Already-registered repositories rediscovered at their known path.
    pub existing: usize,
    /// Registry entries currently in the `Missing` state (after this pass).
    pub missing: usize,
}

/// Reconcile `discovered` (from one scan) into `workspace.repositories`.
pub fn reconcile(workspace: &mut Workspace, discovered: Vec<DiscoveredRepo>) -> ReconcileReport {
    let mut report = ReconcileReport {
        found: discovered.len(),
        ..Default::default()
    };

    // 1. Refresh entries rediscovered at their known path; hold the rest as
    //    candidates for re-link or fresh registration.
    let mut new_candidates: Vec<DiscoveredRepo> = Vec::new();
    for disc in discovered {
        if let Some(repo) = workspace
            .repositories
            .iter_mut()
            .find(|r| r.path == disc.path)
        {
            repo.state = RepositoryState::Active;
            repo.fingerprint = disc.fingerprint;
            report.existing += 1;
        } else {
            new_candidates.push(disc);
        }
    }

    // 2. Any registry entry whose path no longer exists on disk is Missing.
    for repo in workspace.repositories.iter_mut() {
        if !repo.path.exists() {
            repo.state = RepositoryState::Missing;
        }
    }

    // 3. Collision-safe re-link: only when exactly one Missing entry and
    //    exactly one new candidate share a single non-null fingerprint.
    let missing_by_fp = index_by_fingerprint(
        workspace
            .repositories
            .iter()
            .enumerate()
            .filter(|(_, r)| r.state == RepositoryState::Missing)
            .filter_map(|(idx, r)| r.fingerprint.clone().map(|fp| (fp, idx))),
    );
    let new_by_fp = index_by_fingerprint(
        new_candidates
            .iter()
            .enumerate()
            .filter_map(|(idx, d)| d.fingerprint.clone().map(|fp| (fp, idx))),
    );

    let mut relinked_new = vec![false; new_candidates.len()];
    for (fp, missing_idxs) in &missing_by_fp {
        if missing_idxs.len() != 1 {
            continue; // ambiguous on the registry side
        }
        let Some(new_idxs) = new_by_fp.get(fp) else {
            continue;
        };
        if new_idxs.len() != 1 {
            continue; // ambiguous on the discovery side (clones/forks)
        }
        let m = missing_idxs[0];
        let n = new_idxs[0];
        workspace.repositories[m].path = new_candidates[n].path.clone();
        workspace.repositories[m].fingerprint = new_candidates[n].fingerprint.clone();
        workspace.repositories[m].state = RepositoryState::Active;
        relinked_new[n] = true;
        report.relinked += 1;
    }

    // 4. Register the remaining candidates fresh (assigned to Ungrouped).
    let ungrouped_id = workspace.ensure_ungrouped();
    for (idx, disc) in new_candidates.into_iter().enumerate() {
        if relinked_new[idx] {
            continue;
        }
        let mut repo = Repository::new(disc.path, disc.fingerprint);
        repo.group_id = Some(ungrouped_id);
        workspace.repositories.push(repo);
        report.new += 1;
    }

    report.missing = workspace
        .repositories
        .iter()
        .filter(|r| r.state == RepositoryState::Missing)
        .count();
    report
}

fn index_by_fingerprint<I>(items: I) -> HashMap<String, Vec<usize>>
where
    I: IntoIterator<Item = (String, usize)>,
{
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    for (fp, idx) in items {
        map.entry(fp).or_default().push(idx);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn disc(path: PathBuf, fp: Option<&str>) -> DiscoveredRepo {
        DiscoveredRepo {
            path,
            fingerprint: fp.map(str::to_string),
        }
    }

    #[test]
    fn registers_new_then_is_idempotent() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        std::fs::create_dir_all(&a).unwrap();

        let mut ws = Workspace::default();
        let r1 = reconcile(&mut ws, vec![disc(a.clone(), Some("F1"))]);
        assert_eq!(r1.new, 1);
        assert_eq!(ws.repositories.len(), 1);

        let r2 = reconcile(&mut ws, vec![disc(a.clone(), Some("F1"))]);
        assert_eq!(r2.new, 0);
        assert_eq!(r2.existing, 1);
        assert_eq!(ws.repositories.len(), 1);
    }

    #[test]
    fn unique_match_relinks_and_preserves_id() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::create_dir_all(&a).unwrap();

        let mut ws = Workspace::default();
        reconcile(&mut ws, vec![disc(a.clone(), Some("F1"))]);
        let id = ws.repositories[0].id;

        std::fs::remove_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let report = reconcile(&mut ws, vec![disc(b.clone(), Some("F1"))]);

        assert_eq!(report.relinked, 1);
        assert_eq!(ws.repositories.len(), 1);
        assert_eq!(ws.repositories[0].id, id);
        assert_eq!(ws.repositories[0].path, b);
        assert_eq!(ws.repositories[0].state, RepositoryState::Active);
    }

    #[test]
    fn ambiguous_fingerprint_does_not_relink() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();

        let mut ws = Workspace::default();
        reconcile(
            &mut ws,
            vec![disc(a.clone(), Some("F1")), disc(b.clone(), Some("F1"))],
        );
        assert_eq!(ws.repositories.len(), 2);

        std::fs::remove_dir_all(&a).unwrap();
        std::fs::remove_dir_all(&b).unwrap();
        let c = dir.path().join("c");
        let d = dir.path().join("d");
        std::fs::create_dir_all(&c).unwrap();
        std::fs::create_dir_all(&d).unwrap();

        let report = reconcile(&mut ws, vec![disc(c, Some("F1")), disc(d, Some("F1"))]);
        assert_eq!(report.relinked, 0);
        assert_eq!(report.new, 2);
        assert_eq!(report.missing, 2);
        assert_eq!(ws.repositories.len(), 4);
    }

    #[test]
    fn null_fingerprint_never_relinks() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::create_dir_all(&a).unwrap();

        let mut ws = Workspace::default();
        reconcile(&mut ws, vec![disc(a.clone(), None)]);
        std::fs::remove_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();

        let report = reconcile(&mut ws, vec![disc(b, None)]);
        assert_eq!(report.relinked, 0);
        assert_eq!(report.missing, 1);
        assert_eq!(ws.repositories.len(), 2);
    }
}
