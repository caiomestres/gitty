//! Write / network Git operations via shell-out to the `git` CLI (ADR-0001).
//!
//! Read-only inspection stays in [`super::read`] (libgit2). This module owns
//! every operation that touches the network or mutates refs: fetch, pull,
//! checkout. It uses `std::process::Command` with `GIT_TERMINAL_PROMPT=0` and
//! `GIT_SSH_COMMAND="ssh -o BatchMode=yes"` to prevent interactive credential
//! or SSH passphrase prompts from blocking the process.

use std::fmt;
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
// Error classification
// ---------------------------------------------------------------------------

/// Actionable category for a failed `git` operation, derived from stderr
/// pattern matching (D14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Network,
    Auth,
    Conflict,
    DirtyWorkTree,
    BranchNotFound,
    NoUpstream,
    Unknown(String),
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => write!(f, "network error"),
            Self::Auth => write!(f, "authentication failed"),
            Self::Conflict => write!(f, "merge conflict"),
            Self::DirtyWorkTree => write!(f, "dirty working tree"),
            Self::BranchNotFound => write!(f, "branch not found"),
            Self::NoUpstream => write!(f, "no upstream configured"),
            Self::Unknown(msg) => write!(f, "{msg}"),
        }
    }
}

/// Classify stderr from a failed `git` command into an actionable category.
/// Case-insensitive substring matching, first match wins.
pub fn classify_error(stderr: &str) -> ErrorCategory {
    let lower = stderr.to_lowercase();

    if lower.contains("could not resolve host")
        || lower.contains("unable to access")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
        || lower.contains("timed out")
        || lower.contains("connection timed out")
    {
        return ErrorCategory::Network;
    }
    if lower.contains("authentication failed")
        || lower.contains("invalid username or password")
        || lower.contains("could not read username")
        || lower.contains("could not read from remote")
        || lower.contains("terminal prompts disabled")
    {
        return ErrorCategory::Auth;
    }
    if lower.contains("conflict") && lower.contains("merge")
        || lower.contains("fix conflicts")
        || lower.contains("automatic merge failed")
    {
        return ErrorCategory::Conflict;
    }
    if lower.contains("your local changes")
        || lower.contains("please commit your changes or stash them")
        || lower.contains("overwritten by")
    {
        return ErrorCategory::DirtyWorkTree;
    }
    if lower.contains("did not match any")
        || lower.contains("pathspec")
            && (lower.contains("did not match") || lower.contains("unknown revision"))
        || lower.contains("not a valid branch name")
        || lower.contains("invalid reference")
    {
        return ErrorCategory::BranchNotFound;
    }
    if lower.contains("no tracking information")
        || lower.contains("no upstream")
        || lower.contains("there is no tracking information")
    {
        return ErrorCategory::NoUpstream;
    }

    let summary = stderr.lines().next().unwrap_or(stderr).trim().to_string();
    ErrorCategory::Unknown(summary)
}

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
// Repository matching
// ---------------------------------------------------------------------------

/// Match a user-supplied `target` against registered repositories.
///
/// Matches by exact canonical path first, then by last path component
/// (directory name). Returns an error if the directory-name match is
/// ambiguous (multiple repos share the same name).
pub fn match_repo<'a>(
    repos: &'a [Repository],
    target: &str,
) -> std::result::Result<&'a Repository, MatchError> {
    let target_path = Path::new(target);

    // 1. Exact canonical path match.
    if let Some(repo) = repos.iter().find(|r| r.path == target_path) {
        return Ok(repo);
    }

    // 2. Last path component match.
    let matches: Vec<&Repository> = repos
        .iter()
        .filter(|r| {
            r.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name == target)
        })
        .collect();

    match matches.len() {
        0 => Err(MatchError::NotFound(target.to_string())),
        1 => Ok(matches[0]),
        _ => {
            let paths: Vec<String> = matches
                .iter()
                .map(|r| r.path.display().to_string())
                .collect();
            Err(MatchError::Ambiguous {
                name: target.to_string(),
                paths,
            })
        }
    }
}

#[derive(Debug)]
pub enum MatchError {
    NotFound(String),
    Ambiguous { name: String, paths: Vec<String> },
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "no repository matching '{name}'"),
            Self::Ambiguous { name, paths } => {
                write!(f, "'{name}' is ambiguous, matches multiple repositories:")?;
                for p in paths {
                    write!(f, "\n  {p}")?;
                }
                write!(f, "\nUse the full path to disambiguate.")
            }
        }
    }
}

impl std::error::Error for MatchError {}

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
}

/// The operation to run on each repository during batch execution.
pub enum BatchOp<'a> {
    Fetch,
    Pull,
    Checkout(&'a str),
}

impl GitBinary {
    /// Execute `op` against every repository in the list. Sequential in v1
    /// (D15 — parallel requires Lock).
    ///
    /// `Missing` repositories produce `RepoOutcome::Skipped`. Failures never
    /// abort the remaining repositories.
    pub fn run_batch(&self, repos: &[Repository], op: &BatchOp<'_>) -> BatchResult {
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
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- GitBinary resolution --

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

    // -- Error classification --

    #[test]
    fn classify_network_errors() {
        assert_eq!(
            classify_error(
                "fatal: unable to access 'https://...': Could not resolve host: github.com"
            ),
            ErrorCategory::Network,
        );
        assert_eq!(
            classify_error("fatal: unable to access 'https://...': Connection refused"),
            ErrorCategory::Network,
        );
    }

    #[test]
    fn classify_auth_errors() {
        assert_eq!(
            classify_error("fatal: Authentication failed for 'https://...'"),
            ErrorCategory::Auth,
        );
        assert_eq!(
            classify_error(
                "fatal: could not read Username for 'https://...': terminal prompts disabled"
            ),
            ErrorCategory::Auth,
        );
        assert_eq!(
            classify_error("fatal: Could not read from remote repository."),
            ErrorCategory::Auth,
        );
    }

    #[test]
    fn classify_conflict() {
        assert_eq!(
            classify_error("CONFLICT (content): Merge conflict in foo.rs\nAutomatic merge failed; fix conflicts and then commit."),
            ErrorCategory::Conflict,
        );
    }

    #[test]
    fn classify_dirty_worktree() {
        assert_eq!(
            classify_error("error: Your local changes to the following files would be overwritten by checkout:\n  foo.rs\nPlease commit your changes or stash them before you switch branches."),
            ErrorCategory::DirtyWorkTree,
        );
    }

    #[test]
    fn classify_branch_not_found() {
        assert_eq!(
            classify_error(
                "error: pathspec 'no-such-branch' did not match any file(s) known to git"
            ),
            ErrorCategory::BranchNotFound,
        );
    }

    #[test]
    fn classify_no_upstream() {
        assert_eq!(
            classify_error("There is no tracking information for the current branch."),
            ErrorCategory::NoUpstream,
        );
    }

    #[test]
    fn classify_unknown_fallback() {
        let kind = classify_error("something unexpected happened\nsecond line");
        assert!(
            matches!(kind, ErrorCategory::Unknown(msg) if msg == "something unexpected happened")
        );
    }

    // -- Runner + typed operations against a real repo --

    fn init_test_repo(dir: &Path) {
        let repo = git2::Repository::init(dir).unwrap();
        let workdir = repo.workdir().unwrap();
        std::fs::write(workdir.join("a.txt"), "hello").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("a.txt")).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();
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

    // -- Repository matching --

    #[test]
    fn match_repo_by_exact_path() {
        let repos = vec![
            Repository::new(PathBuf::from("/code/alpha"), Some("fp1".into())),
            Repository::new(PathBuf::from("/code/beta"), Some("fp2".into())),
        ];
        let found = match_repo(&repos, "/code/alpha").unwrap();
        assert_eq!(found.path, Path::new("/code/alpha"));
    }

    #[test]
    fn match_repo_by_dir_name() {
        let repos = vec![
            Repository::new(PathBuf::from("/code/alpha"), Some("fp1".into())),
            Repository::new(PathBuf::from("/code/beta"), Some("fp2".into())),
        ];
        let found = match_repo(&repos, "beta").unwrap();
        assert_eq!(found.path, Path::new("/code/beta"));
    }

    #[test]
    fn match_repo_ambiguous_errors() {
        let repos = vec![
            Repository::new(PathBuf::from("/a/shared"), Some("fp1".into())),
            Repository::new(PathBuf::from("/b/shared"), Some("fp2".into())),
        ];
        let err = match_repo(&repos, "shared").unwrap_err();
        assert!(matches!(err, MatchError::Ambiguous { .. }));
    }

    #[test]
    fn match_repo_not_found_errors() {
        let repos = vec![Repository::new(
            PathBuf::from("/code/alpha"),
            Some("fp1".into()),
        )];
        let err = match_repo(&repos, "nope").unwrap_err();
        assert!(matches!(err, MatchError::NotFound(_)));
    }

    // -- Batch execution --

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
