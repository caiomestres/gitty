# Foundation — Git Write Layer Tasks

> Atomic tasks for the `foundation-git-write` feature. `[P]` = parallelizable. Each task lists Done-when + Tests + Gate.
> Build test-first where a behavior is non-trivial (TDD).

## T1 — `CoreError` extension + `GitBinary` resolution

- **What**: Add `GitNotFound` variant to `CoreError`. Implement `GitBinary` struct with `resolve()` → finds `git` on PATH via `git --version`, parses version. Cache-friendly (returns owned struct).
- **Where**: `crates/gitty-core/src/error.rs`, `crates/gitty-core/src/git/write.rs`, update `git/mod.rs`.
- **Depends on**: —
- **Done when**: `GitBinary::resolve()` returns `Ok(GitBinary)` when git is on PATH; returns `Err(CoreError::GitNotFound)` when it isn't.
- **Tests**: unit — `resolve()` succeeds on dev/CI machines (git is always installed); version string is non-empty.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: GWRITE-01

## T2 — Shell-out runner (`GitBinary::run`) [P after T1]

- **What**: `run(&self, repo_path, args) -> Result<GitOutput>`. Spawns `git` with `current_dir`, pipes stdout/stderr, sets `GIT_TERMINAL_PROMPT=0` and `GIT_SSH_COMMAND="ssh -o BatchMode=yes"`. Returns `GitOutput { exit_code, stdout, stderr }`.
- **Where**: `crates/gitty-core/src/git/write.rs`.
- **Depends on**: T1
- **Done when**: can execute `git status` via the runner on a temp repo and get exit code 0 + stdout output.
- **Tests**: integration — init a temp repo, run `git status` through the runner, assert exit_code=0 and stdout contains branch info.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: GWRITE-02

## T3 — Exit code classification [P after T1]

- **What**: `classify_error(&GitOutput) -> ErrorCategory`. Substring matching on stderr for Network, Conflict, Auth, BranchNotFound, DirtyWorkTree, NoUpstream, Unknown.
- **Where**: `crates/gitty-core/src/git/write.rs`.
- **Depends on**: T1 (for `GitOutput` type)
- **Done when**: synthetic `GitOutput` values with known stderr patterns are classified correctly.
- **Tests**: unit — one test per category: construct a `GitOutput` with the matching stderr pattern, assert correct `ErrorCategory`. Plus a test for `Unknown` fallthrough.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: GWRITE-03

## T4 — Typed operations: fetch, pull, checkout [P after T2]

- **What**: `GitBinary::fetch(path)`, `GitBinary::pull(path)`, `GitBinary::checkout(path, branch)`. Each calls `run()` with the right args, wraps result in `GitResult` (Success or Failed + category).
- **Where**: `crates/gitty-core/src/git/write.rs`.
- **Depends on**: T2, T3
- **Done when**: fetch/pull/checkout succeed on temp repos with local file-protocol remotes.
- **Tests**: integration `crates/gitty-core/tests/git_write.rs` — test fixture creates a bare remote + local clone + contributor push; fetch sees updates, pull merges them, checkout switches branch.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: GWRITE-04, GWRITE-05, GWRITE-06

## T5 — Batch execution + repo matching

- **What**: `run_batch()` function that iterates repos, skips Missing, calls an operation, collects `BatchResult`. Repository matching helper: match by full path or last component, error on ambiguity.
- **Where**: `crates/gitty-core/src/git/write.rs` (batch), `crates/gitty-core/src/repository.rs` (matching helper).
- **Depends on**: T4
- **Done when**: batch over 3 repos (1 active, 1 active, 1 missing) produces 2 Success + 1 Skipped. Matching by dir name works; ambiguous match returns error.
- **Tests**: unit/integration — batch with mixed states; matching by name vs path; ambiguity error.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: GWRITE-10, GWRITE-11

## T6 — CLI `fetch` / `pull` / `checkout` commands

- **What**: clap subcommands: `fetch [path]`, `pull [path]`, `checkout <branch> [--repo <path>]`. Resolve git binary, load config, run batch or single, print per-repo results with status symbols (✓/✗/⊘).
- **Where**: `crates/gitty-cli/src/main.rs`.
- **Depends on**: T5
- **Done when**: `gitty fetch`, `gitty pull`, `gitty checkout dev` work end-to-end on temp repos with local remotes.
- **Tests**: `crates/gitty-cli/tests/cli_write.rs` via `assert_cmd` — scan a temp tree with a local remote, run fetch/pull/checkout, assert output patterns.
- **Gate**: `cargo test`
- **Covers**: GWRITE-07, GWRITE-08, GWRITE-09

## T7 — Final gate & cleanup

- **What**: Ensure no clippy warnings, fmt clean. Remove any TODO/FIXME markers. Update `git/mod.rs` exports.
- **Depends on**: T6
- **Done when**: full gate green.
- **Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- **Covers**: Success Criteria

## Traceability (tasks ↔ requirements)

| Req | Task(s) |
| --- | --- |
| GWRITE-01 | T1 |
| GWRITE-02 | T2 |
| GWRITE-03 | T3 |
| GWRITE-04 | T4 |
| GWRITE-05 | T4 |
| GWRITE-06 | T4 |
| GWRITE-07 | T6 |
| GWRITE-08 | T6 |
| GWRITE-09 | T6 |
| GWRITE-10 | T5 |
| GWRITE-11 | T5 |

**Coverage:** 11/11 requirements mapped.

## Suggested commit sequence (atomic)

1. `feat(core): git binary resolution and shell-out runner` — T1, T2
2. `feat(core): exit code classification for git operations` — T3
3. `feat(core): typed git write operations (fetch, pull, checkout)` — T4
4. `feat(core): batch execution and repository matching` — T5
5. `feat(cli): fetch, pull, checkout commands` — T6
6. `chore: clippy/fmt cleanup` — T7
