mod common;

use std::fs;

use common::{commit_file, init_repo};
use gitty_core::scan;
use tempfile::tempdir;

#[test]
fn discovers_nested_repos_and_skips_ignored_dirs() {
    let root = tempdir().unwrap();
    let base = root.path();

    // repoA at base/repoA
    let repo_a = base.join("repoA");
    fs::create_dir_all(&repo_a).unwrap();
    let ra = init_repo(&repo_a);
    commit_file(&ra, "f.txt", "x", "init");

    // Decoy: an ignored directory containing something that looks like a repo.
    let decoy = repo_a.join("node_modules").join("pkg");
    fs::create_dir_all(&decoy).unwrap();
    fs::create_dir_all(decoy.join(".git")).unwrap();

    // Nested repoB at base/repoA/sub/repoB (must still be discovered, D8).
    let repo_b = repo_a.join("sub").join("repoB");
    fs::create_dir_all(&repo_b).unwrap();
    let rb = init_repo(&repo_b);
    commit_file(&rb, "g.txt", "y", "init");

    // A plain, non-repo directory.
    fs::create_dir_all(base.join("plain")).unwrap();

    let found = scan::scan(base).unwrap();
    let paths: Vec<_> = found.iter().map(|d| d.path.clone()).collect();

    assert_eq!(
        found.len(),
        2,
        "should find repoA and nested repoB, never the node_modules decoy"
    );
    assert!(paths.iter().any(|p| p.ends_with("repoA")));
    assert!(paths.iter().any(|p| p.ends_with("repoB")));
}

#[test]
fn scan_missing_path_errors() {
    let root = tempdir().unwrap();
    let missing = root.path().join("does-not-exist");
    assert!(scan::scan(&missing).is_err());
}

#[test]
fn empty_tree_finds_nothing() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("a").join("b").join("c")).unwrap();
    let found = scan::scan(root.path()).unwrap();
    assert!(found.is_empty());
}
