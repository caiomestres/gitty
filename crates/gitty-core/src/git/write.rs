//! Write / network Git operations via shell-out to the `git` CLI (ADR-0001).
//!
//! Read-only inspection stays in [`super::read`] (libgit2). This module owns
//! every operation that touches the network or mutates refs: fetch, pull,
//! checkout. It uses `std::process::Command` with `GIT_TERMINAL_PROMPT=0` and
//! `GIT_SSH_COMMAND="ssh -o BatchMode=yes"` to prevent interactive credential
//! or SSH passphrase prompts from blocking the process.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{CoreError, Result};
use crate::repository::{Repository, RepositoryState};

// ---------------------------------------------------------------------------
// GitBinary
// ---------------------------------------------------------------------------

/// A validated path to the `git` executable, with its version string.
#[derive(Debug, Clone)]
pub struct GitBinary {
    path: PathBuf,
    version: String,
}

impl GitBinary {
    /// Locate `git` on `PATH`, run `git --version`, and parse the version.
    ///
    /// Returns `Err(CoreError::GitNotFound)` if `git` is not on PATH or the
    /// version check fails. An unparseable version string is accepted with
    /// the raw stdout stored (non-blocking per spec GWRITE-01 AC4).
    pub fn resolve() -> Result<Self> {
        let name = if cfg!(windows) { "git.exe" } else { "git" };
        let path = which(name).ok_or(CoreError::GitNotFound)?;

        let output = Command::new(&path)
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .map_err(|_| CoreError::GitNotFound)?;

        if !output.status.success() {
            return Err(CoreError::GitNotFound);
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Self { path, version })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    #[cfg(test)]
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            path,
            version: "test".into(),
        }
    }
}

/// Minimal PATH-based lookup (no extra dependency).
fn which(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(binary))
        .find(|p| p.is_file())
}

// ---------------------------------------------------------------------------
// Shell-out runner
// ---------------------------------------------------------------------------

/// Raw output from a `git` invocation.
#[derive(Debug, Clone)]
pub struct GitOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl GitBinary {
    /// Run `git <args>` inside `repo_dir` with interactive prompts disabled.
    pub fn run(&self, repo_dir: &Path, args: &[&str]) -> Result<GitOutput> {
        let output = Command::new(&self.path)
            .args(args)
            .current_dir(repo_dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()?;

        Ok(GitOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

// ---------------------------------------------------------------------------
// Error classification (re-exported from git::classify)
// ---------------------------------------------------------------------------

pub use super::classify::{classify_error, ErrorCategory};

/// Classified outcome of a git write operation.
#[derive(Debug)]
pub enum GitResult {
    Success(GitOutput),
    Failed {
        output: GitOutput,
        category: ErrorCategory,
    },
}

impl GitResult {
    /// Convert raw `GitOutput` into a classified `GitResult`.
    pub fn from_output(output: GitOutput) -> Self {
        if output.exit_code == 0 {
            GitResult::Success(output)
        } else {
            let category = classify_error(&output.stderr);
            GitResult::Failed { output, category }
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, GitResult::Success(_))
    }
}

// ---------------------------------------------------------------------------
// Typed operations
// ---------------------------------------------------------------------------

impl GitBinary {
    /// `git fetch --all` inside a repository.
    pub fn fetch(&self, repo_dir: &Path) -> Result<GitResult> {
        let output = self.run(repo_dir, &["fetch", "--all"])?;
        Ok(GitResult::from_output(output))
    }

    /// `git pull` inside a repository (uses repo's default upstream).
    pub fn pull(&self, repo_dir: &Path) -> Result<GitResult> {
        let output = self.run(repo_dir, &["pull"])?;
        Ok(GitResult::from_output(output))
    }

    /// `git checkout <branch>` inside a repository.
    pub fn checkout(&self, repo_dir: &Path, branch: &str) -> Result<GitResult> {
        let output = self.run(repo_dir, &["checkout", branch])?;
        Ok(GitResult::from_output(output))
    }
}

// ---------------------------------------------------------------------------
// Repository matching (re-exported from git::resolve)
// ---------------------------------------------------------------------------

pub use super::resolve::{match_repo, MatchError};

// ---------------------------------------------------------------------------
// Batch execution
// ---------------------------------------------------------------------------

/// Outcome of a single repository within a batch run.
#[derive(Debug)]
pub struct RepoOperationResult {
    pub repo_path: PathBuf,
    pub outcome: RepoOutcome,
}

/// Per-repository outcome in a batch operation.
#[derive(Debug)]
pub enum RepoOutcome {
    Success(GitOutput),
    Failed {
        output: GitOutput,
        category: ErrorCategory,
    },
    Skipped {
        reason: String,
    },
    Locked {
        holder_pid: u32,
        since: String,
    },
}

/// Aggregated result of running a git operation across multiple repositories.
#[derive(Debug)]
pub struct BatchResult {
    pub results: Vec<RepoOperationResult>,
}

impl BatchResult {
    pub fn success_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, RepoOutcome::Success(_)))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, RepoOutcome::Failed { .. }))
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, RepoOutcome::Skipped { .. }))
            .count()
    }

    pub fn locked_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.outcome, RepoOutcome::Locked { .. }))
            .count()
    }
}

/// The operation to run on each repository during batch execution.
pub enum BatchOp<'a> {
    Fetch,
    Pull,
    Checkout(&'a str),
}

impl GitBinary {
    /// Execute `op` against every repository in the list, optionally acquiring
    /// per-repo locks when `locks_dir` is provided (ADR-0006).
    ///
    /// `Missing` repositories produce `RepoOutcome::Skipped`. Failures never
    /// abort the remaining repositories.
    pub fn run_batch_in(
        &self,
        repos: &[Repository],
        op: &BatchOp<'_>,
        locks_dir: Option<&Path>,
    ) -> BatchResult {
        use crate::lock::RepoLock;

        let results = repos
            .iter()
            .map(|r| {
                if r.state == RepositoryState::Missing {
                    return RepoOperationResult {
                        repo_path: r.path.clone(),
                        outcome: RepoOutcome::Skipped {
                            reason: "repository path not found".into(),
                        },
                    };
                }

                let _lock = if let Some(dir) = locks_dir {
                    match RepoLock::acquire_in(r.id, dir) {
                        Ok(lock) => Some(lock),
                        Err(CoreError::LockContention { pid, since, .. }) => {
                            return RepoOperationResult {
                                repo_path: r.path.clone(),
                                outcome: RepoOutcome::Locked {
                                    holder_pid: pid,
                                    since,
                                },
                            };
                        }
                        Err(e) => {
                            return RepoOperationResult {
                                repo_path: r.path.clone(),
                                outcome: RepoOutcome::Failed {
                                    output: GitOutput {
                                        exit_code: -1,
                                        stdout: String::new(),
                                        stderr: format!("lock error: {e}"),
                                    },
                                    category: ErrorCategory::Unknown(format!("lock error: {e}")),
                                },
                            };
                        }
                    }
                } else {
                    None
                };

                let git_result = match op {
                    BatchOp::Fetch => self.fetch(&r.path),
                    BatchOp::Pull => self.pull(&r.path),
                    BatchOp::Checkout(branch) => self.checkout(&r.path, branch),
                };

                let outcome = match git_result {
                    Ok(GitResult::Success(output)) => RepoOutcome::Success(output),
                    Ok(GitResult::Failed { output, category }) => {
                        RepoOutcome::Failed { output, category }
                    }
                    Err(e) => RepoOutcome::Failed {
                        output: GitOutput {
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: e.to_string(),
                        },
                        category: ErrorCategory::Unknown(e.to_string()),
                    },
                };

                RepoOperationResult {
                    repo_path: r.path.clone(),
                    outcome,
                }
            })
            .collect();

        BatchResult { results }
    }

    /// Execute `op` without locking (convenience wrapper).
    pub fn run_batch(&self, repos: &[Repository], op: &BatchOp<'_>) -> BatchResult {
        self.run_batch_in(repos, op, None)
    }

    /// Execute `op` with per-repo locking using the default locks directory.
    pub fn run_batch_locked(&self, repos: &[Repository], op: &BatchOp<'_>) -> Result<BatchResult> {
        let locks_dir = crate::config::paths::locks_dir()?;
        Ok(self.run_batch_in(repos, op, Some(&locks_dir)))
    }

    /// Locked batch execution with a caller-specified locks directory.
    pub fn run_batch_locked_in(
        &self,
        repos: &[Repository],
        op: &BatchOp<'_>,
        locks_dir: &Path,
    ) -> BatchResult {
        self.run_batch_in(repos, op, Some(locks_dir))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_helpers::init_test_repo;

    #[test]
    fn resolve_finds_git_on_path() {
        let git = GitBinary::resolve();
        assert!(git.is_ok(), "git should be on PATH in CI and dev");
    }

    #[test]
    fn resolve_stores_version_string() {
        let git = GitBinary::resolve().unwrap();
        assert!(
            git.version().starts_with("git version"),
            "version should start with 'git version', got: {}",
            git.version()
        );
    }

    #[test]
    fn run_returns_exit_code_and_stdout() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());

        let git = GitBinary::resolve().unwrap();
        let out = git.run(dir.path(), &["status", "--porcelain"]).unwrap();
        assert_eq!(out.exit_code, 0);
    }

    #[test]
    fn fetch_on_local_repo_without_remote_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());

        let git = GitBinary::resolve().unwrap();
        let result = git.fetch(dir.path()).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn checkout_nonexistent_branch_classifies_as_branch_not_found() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());

        let git = GitBinary::resolve().unwrap();
        let result = git.checkout(dir.path(), "no-such-branch").unwrap();
        match result {
            GitResult::Failed { category, .. } => {
                assert_eq!(category, ErrorCategory::BranchNotFound)
            }
            GitResult::Success(_) => panic!("expected Failed, got Success"),
        }
    }

    #[test]
    fn batch_includes_skipped_for_missing_repos() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());

        let active = Repository::new(dir.path().to_path_buf(), Some("fp".into()));
        let mut missing = Repository::new(PathBuf::from("/tmp/nonexistent"), None);
        missing.state = RepositoryState::Missing;

        let git = GitBinary::resolve().unwrap();
        let batch = git.run_batch(&[active, missing], &BatchOp::Fetch);

        assert_eq!(batch.results.len(), 2);
        assert!(matches!(batch.results[0].outcome, RepoOutcome::Success(_)));
        assert!(matches!(
            batch.results[1].outcome,
            RepoOutcome::Skipped { .. }
        ));
        assert_eq!(batch.success_count(), 1);
        assert_eq!(batch.skipped_count(), 1);
        assert_eq!(batch.failed_count(), 0);
    }
}
