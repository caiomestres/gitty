# Foundation — Git Write Layer Design

> Implements `spec.md`. The shell-out approach is already locked in ADR-0001.
> This doc fixes the module layout, data shapes, and runner design before coding.

## Crate & module layout (`gitty-core`)

```
crates/gitty-core/src/
├── ...existing modules...
└── git/
    ├── mod.rs        # re-exports read + write
    ├── read.rs       # (existing) git2-backed status, fingerprint, etc.
    └── write.rs      # shell-out runner, git binary resolution, typed operations
```

No new crates. The write layer lives alongside the read layer under `git/`, both re-exported through `git/mod.rs`.

## Data structures

```rust
// git/write.rs

/// Result of a single git shell-out execution.
pub struct GitOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Validated git binary location + version.
pub struct GitBinary {
    path: PathBuf,
    version: String,
}

/// Classified outcome of a git write operation.
pub enum GitResult {
    Success(GitOutput),
    Failed {
        output: GitOutput,
        category: ErrorCategory,
    },
}

/// Actionable error categories parsed from stderr/exit-code patterns.
pub enum ErrorCategory {
    /// Network unreachable, DNS failure, connection refused/timed out.
    Network,
    /// Merge conflict during pull or rebase.
    Conflict,
    /// Authentication failure (SSH key, credential helper).
    Auth,
    /// Target branch does not exist in the repository.
    BranchNotFound,
    /// Working tree has uncommitted changes preventing the operation.
    DirtyWorkTree,
    /// No upstream/tracking branch configured.
    NoUpstream,
    /// Unclassified failure — stderr + exit code available for the caller.
    Unknown,
}

/// The result of running a git operation across multiple repositories.
pub struct BatchResult {
    pub results: Vec<RepoOperationResult>,
}

pub struct RepoOperationResult {
    pub repo_path: PathBuf,
    pub outcome: RepoOutcome,
}

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
```

## Key algorithms

### Git binary resolution (`GitBinary::resolve`)

1. Run `which git` (Unix) / `where git` (Windows) — or use `std::process::Command::new("git")` 
   and let the OS resolve PATH. Simpler: just run `git --version` directly.
2. If the command fails to spawn → `CoreError::GitNotFound`.
3. Parse stdout: extract version via regex `git version (\d+\.\d+.*)`. 
   If parse fails → warn (non-blocking), store raw string.
4. Cache the result for the process lifetime (no need to re-resolve per operation).

Implementation note: use `std::process::Command` directly. No dependency on `which` crate —
`Command::new("git")` already searches PATH on all platforms.

### Shell-out runner (`GitBinary::run`)

```
fn run(&self, repo_path: &Path, args: &[&str]) -> Result<GitOutput>
```

1. Build `Command::new(&self.path)` with `args`.
2. Set `current_dir(repo_path)`.
3. Set `stdout(Stdio::piped())`, `stderr(Stdio::piped())`.
4. Disable interactive prompts: set env `GIT_TERMINAL_PROMPT=0` (prevents credential popups)
   and `GIT_SSH_COMMAND="ssh -o BatchMode=yes"` (prevents SSH passphrase prompts).
5. `spawn()` + `wait_with_output()`.
6. Capture exit code, stdout, stderr into `GitOutput`.
7. I/O errors (command not found, permission denied) → `CoreError::Io`.

### Exit code classification (`classify_error`)

Parses `GitOutput` (exit code + stderr) into an `ErrorCategory`:

| Pattern (stderr contains) | Category |
| --- | --- |
| `fatal: Could not read from remote` | Auth |
| `Authentication failed` | Auth |
| `fatal: unable to access` + `Could not resolve host` | Network |
| `fatal: unable to access` + `Failed to connect` | Network |
| `Connection timed out` / `Connection refused` | Network |
| `CONFLICT` or `Automatic merge failed` | Conflict |
| `error: Your local changes` | DirtyWorkTree |
| `error: pathspec .* did not match` | BranchNotFound |
| `There is no tracking information` | NoUpstream |
| _(no match)_ | Unknown |

Case-insensitive substring matching, checked in order. First match wins.

### Typed operations

Each operation is a thin wrapper calling `GitBinary::run` with the right args:

| Operation | Git command | Notes |
| --- | --- | --- |
| `fetch(repo_path)` | `git fetch --all` | Fetches all remotes |
| `pull(repo_path)` | `git pull` | Uses repo's default upstream |
| `checkout(repo_path, branch)` | `git checkout <branch>` | Switches branch |

Each returns `GitResult` (Success or Failed with category).

### Batch execution

```
fn run_batch(
    git: &GitBinary,
    repos: &[&Repository],
    operation: impl Fn(&GitBinary, &Path) -> Result<GitResult>,
) -> BatchResult
```

1. Filter: skip repos with `state == Missing` → `RepoOutcome::Skipped`.
2. For each active repo: call `operation(git, &repo.path)`.
3. Collect all results — never short-circuit on failure.
4. Return `BatchResult`.

Sequential execution in v1 (parallelism is a later optimization, and requires the Lock feature).

## CLI surface (`gitty-cli`)

| Command | Behavior |
| --- | --- |
| `gitty fetch` | Resolve git binary, load config, fetch all active repos. Print per-repo result. |
| `gitty fetch <path>` | Fetch only the repo matching `<path>` (by last path component or full path). |
| `gitty pull` | Pull all active repos. Print per-repo result. |
| `gitty pull <path>` | Pull only the matching repo. |
| `gitty checkout <branch>` | Checkout `<branch>` on all active repos. Print per-repo result. |
| `gitty checkout <branch> --repo <path>` | Checkout on the matching repo only. |

Per-repo output format:

```
✓ my-repo               fetched successfully
✗ other-repo             [network] fatal: unable to access '...': Could not resolve host
⊘ missing-repo           [skipped] repository path not found
```

CLI resolves the git binary once at startup and reuses it for all operations.

### Repository matching

The `<path>` argument matches against registered repositories by:
1. Exact canonical path match.
2. Last path component (directory name) match — e.g., `my-repo` matches `/home/user/code/my-repo`.
3. If ambiguous (multiple repos share the same directory name) → error asking user to use the full path.

## New dependencies

None. `std::process::Command` is sufficient for the shell-out runner.

## Test strategy

- **gitty-core unit**: test `classify_error` with synthetic `GitOutput` values covering each category.
- **gitty-core unit**: test `GitBinary::resolve` succeeds when git is on PATH (CI and dev machines).
- **gitty-core integration**: create temp repos with local file-protocol remotes;
  run fetch/pull/checkout; assert outcomes.
- **gitty-cli integration**: invoke `gitty fetch`, `gitty pull`, `gitty checkout` via `assert_cmd`
  against temp repos with local remotes.
- Gate: `cargo test` + `cargo clippy -- -D warnings`.

### Test fixture: local remote repos

To test fetch/pull without network access, create paired repos:
1. Init "remote" bare repo (`git init --bare`).
2. Clone it to a "local" repo.
3. Push a commit from a third "contributor" clone to the bare remote.
4. Now `fetch`/`pull` from the "local" repo will have something to fetch.

This uses the `file://` protocol — no network required.
