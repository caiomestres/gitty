//! End-to-end CLI tests. Each test points `GITTY_CONFIG_DIR` at a temp dir so
//! it never touches the real user config.

use std::path::Path;

use assert_cmd::Command;
use git2::{Repository, Signature};
use tempfile::tempdir;

fn init_repo_with_commit(path: &Path) {
    let repo = Repository::init(path).unwrap();
    std::fs::write(path.join("README.md"), "hello").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .unwrap();
}

fn gitty(config_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("gitty").unwrap();
    cmd.env("GITTY_CONFIG_DIR", config_dir);
    cmd
}

#[test]
fn scan_list_status_end_to_end() {
    let config_dir = tempdir().unwrap();
    let work = tempdir().unwrap();

    // Two real repos under the scan root.
    let repo_one = work.path().join("alpha");
    let repo_two = work.path().join("nested").join("beta");
    std::fs::create_dir_all(&repo_one).unwrap();
    std::fs::create_dir_all(&repo_two).unwrap();
    init_repo_with_commit(&repo_one);
    init_repo_with_commit(&repo_two);

    // scan
    gitty(config_dir.path())
        .arg("scan")
        .arg(work.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("2 new"));

    // list shows both repos
    gitty(config_dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicates::str::contains("alpha"))
        .stdout(predicates::str::contains("beta"));

    // status shows a clean repo on its branch
    gitty(config_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicates::str::contains("clean"));
}

#[test]
fn list_empty_state_is_friendly() {
    let config_dir = tempdir().unwrap();
    gitty(config_dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicates::str::contains("No repositories tracked yet"));
}

#[test]
fn rescan_is_idempotent() {
    let config_dir = tempdir().unwrap();
    let work = tempdir().unwrap();
    let repo = work.path().join("alpha");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo_with_commit(&repo);

    gitty(config_dir.path())
        .args(["scan", work.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("1 new"));

    // Second scan: nothing new.
    gitty(config_dir.path())
        .args(["scan", work.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("0 new"));
}
