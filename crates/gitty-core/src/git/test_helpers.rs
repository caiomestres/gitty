//! Shared test utilities for git-related tests.

use std::path::Path;

/// Initialize a minimal git repository with one committed file.
pub fn init_test_repo(dir: &Path) {
    let repo = git2::Repository::init(dir).unwrap();
    let workdir = repo.workdir().unwrap();
    std::fs::write(workdir.join("a.txt"), "hello").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("a.txt")).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
}
