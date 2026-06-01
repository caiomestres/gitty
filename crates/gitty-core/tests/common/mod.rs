#![allow(dead_code)] // shared across multiple integration test binaries

use std::path::Path;

use git2::{Repository, Signature};

/// Initialise a fresh Git repository at `path`.
pub fn init_repo(path: &Path) -> Repository {
    Repository::init(path).expect("init repo")
}

/// Stage and commit a single file, creating the initial commit when needed.
pub fn commit_file(repo: &Repository, name: &str, content: &str, message: &str) -> git2::Oid {
    let workdir = repo.workdir().expect("repo has a working directory");
    std::fs::write(workdir.join(name), content).expect("write file");

    let mut index = repo.index().expect("open index");
    index.add_path(Path::new(name)).expect("stage file");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");

    let sig = Signature::now("Test", "test@example.com").expect("signature");
    let parent = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .expect("commit")
}
