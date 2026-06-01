mod common;

use common::{commit_file, init_repo};
use gitty_core::git::read::{self, ChangeStatus};
use tempfile::tempdir;

#[test]
fn empty_repo_has_no_fingerprint_or_head() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());

    assert_eq!(read::root_fingerprint(dir.path()).unwrap(), None);

    let status = read::read_status(dir.path()).unwrap();
    assert!(status.head.is_none());
    assert!(!status.dirty);
}

#[test]
fn single_commit_exposes_head_and_fingerprint() {
    let dir = tempdir().unwrap();
    let repo = init_repo(dir.path());
    commit_file(&repo, "a.txt", "hello", "initial commit");

    assert!(read::root_fingerprint(dir.path()).unwrap().is_some());

    let status = read::read_status(dir.path()).unwrap();
    let head = status.head.expect("head present");
    assert_eq!(head.subject, "initial commit");
    assert!(!head.author.is_empty());
    assert!(status.branch.is_some());
    assert!(!status.dirty);
}

#[test]
fn modified_file_is_dirty() {
    let dir = tempdir().unwrap();
    let repo = init_repo(dir.path());
    commit_file(&repo, "a.txt", "hello", "init");
    std::fs::write(dir.path().join("a.txt"), "changed").unwrap();

    let status = read::read_status(dir.path()).unwrap();
    assert!(status.dirty);
    assert!(status
        .changed_files
        .iter()
        .any(|f| f.path == "a.txt" && f.status == ChangeStatus::Modified));
}

#[test]
fn untracked_file_is_reported() {
    let dir = tempdir().unwrap();
    let repo = init_repo(dir.path());
    commit_file(&repo, "a.txt", "hello", "init");
    std::fs::write(dir.path().join("new.txt"), "x").unwrap();

    let status = read::read_status(dir.path()).unwrap();
    assert!(status
        .changed_files
        .iter()
        .any(|f| f.path == "new.txt" && f.status == ChangeStatus::Untracked));
}

#[test]
fn detached_head_reports_no_branch() {
    let dir = tempdir().unwrap();
    let repo = init_repo(dir.path());
    let oid = commit_file(&repo, "a.txt", "hello", "init");
    repo.set_head_detached(oid).unwrap();

    let status = read::read_status(dir.path()).unwrap();
    assert!(status.detached);
    assert!(status.branch.is_none());
}

#[test]
fn no_upstream_when_not_configured() {
    let dir = tempdir().unwrap();
    let repo = init_repo(dir.path());
    commit_file(&repo, "a.txt", "hello", "init");

    let status = read::read_status(dir.path()).unwrap();
    assert!(status.upstream.is_none());
}

#[test]
fn ahead_behind_computed_against_upstream() {
    let origin = tempdir().unwrap();
    let repo = init_repo(origin.path());
    commit_file(&repo, "a.txt", "hello", "init");

    let work = tempdir().unwrap();
    let work_path = work.path().join("clone");
    let cloned = git2::Repository::clone(origin.path().to_str().unwrap(), &work_path).unwrap();

    // One local commit not in origin -> ahead by 1.
    commit_file(&cloned, "b.txt", "more", "second");

    let status = read::read_status(&work_path).unwrap();
    let up = status.upstream.expect("upstream present after clone");
    assert_eq!(up.ahead, 1);
    assert_eq!(up.behind, 0);
}
