# Foundation — Discovery & Registry Tasks

> Atomic tasks for the `foundation-discovery` feature. `[P]` = parallelizable. Each task lists Done-when + Tests + Gate.
> Build test-first where a behavior is non-trivial (TDD).

## T1 — Add dependencies & verify git2 builds (Windows)

- **What**: Add `dirs`, `walkdir`, `git2`, `thiserror` to `gitty-core`; `anyhow` to `gitty-cli`; `tempfile` + `assert_cmd` as dev-deps. Add shared versions to workspace deps where sensible.
- **Where**: `Cargo.toml` (root), `crates/gitty-core/Cargo.toml`, `crates/gitty-cli/Cargo.toml`.
- **Depends on**: —
- **Done when**: `cargo build` succeeds on Windows (confirms vendored libgit2 compiles).
- **Gate**: `cargo build`
- **Covers**: enabling DISC-*

## T2 — `error` module

- **What**: `CoreError` enum (`Io`, `Json`, `UnsupportedSchema`, `Git`, `PathNotFound`) + `Result<T>` alias.
- **Where**: `crates/gitty-core/src/error.rs`, re-export in `lib.rs`.
- **Depends on**: T1
- **Done when**: compiles; `From` impls for `std::io::Error`, `serde_json::Error`, `git2::Error`.
- **Gate**: `cargo build -p gitty-core`
- **Covers**: cross-cutting

## T3 — `config` module (schema, paths, load/save) [P after T2]

- **What**: `Config`/`Workspace`/`ScanRoot` structs, `CURRENT_SCHEMA_VERSION=1`, `paths.rs` (config dir/file/locks via `dirs`), `load()`/`save()` (atomic write), schema-version check.
- **Where**: `crates/gitty-core/src/config/{mod.rs,paths.rs}`.
- **Depends on**: T2
- **Done when**: round-trip test passes; unknown version → `UnsupportedSchema`; missing file → default.
- **Tests**: unit — serialize/deserialize round-trip; version-mismatch error; (use a temp path override for save).
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-05

## T4 — `repository` model (Repository, state, Registry) [P after T2]

- **What**: `Repository`, `RepositoryState`, registry helpers on `Workspace` (find by path/id, dedupe). Pre-model `group_id`/`tags` defaulted.
- **Where**: `crates/gitty-core/src/repository.rs`.
- **Depends on**: T2
- **Done when**: serde round-trips; helpers unit-tested.
- **Tests**: unit — add/find/dedupe by canonical path.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-02

## T5 — `git::read` (status, fingerprint, HEAD, changed files)

- **What**: `RepositoryStatus` + friends; `read_status(path)`, `root_fingerprint(path)`.
- **Where**: `crates/gitty-core/src/git/{mod.rs,read.rs}`.
- **Depends on**: T2
- **Done when**: against fixture repos — clean repo, dirty repo, branch, detached, ahead/behind, no-commit repo all return correct values.
- **Tests**: integration `crates/gitty-core/tests/git_read.rs` with a fixture builder (init/commit/branch/remote).
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-07, DISC-11, DISC-12, DISC-02 (fingerprint)

## T6 — `scan` (walkdir discovery)

- **What**: `scan(root) -> Vec<DiscoveredRepo>`; prune `.git` + ignore list; no symlink follow; dedupe by canonical path; fingerprint via T5.
- **Where**: `crates/gitty-core/src/scan.rs`.
- **Depends on**: T5
- **Done when**: temp tree with nested repos + `node_modules` decoy + symlink yields exactly the expected repos.
- **Tests**: integration `crates/gitty-core/tests/scan.rs`.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-01, DISC-03

## T7 — `reconcile` (idempotency, missing, collision-safe re-link)

- **What**: `reconcile(&mut Workspace, root, Vec<DiscoveredRepo>) -> ReconcileReport`.
- **Where**: `crates/gitty-core/src/reconcile.rs`.
- **Depends on**: T4, T6
- **Done when**: idempotent rescan adds no dupes; vanished path → Missing; single-match relink works; collision/null → no relink.
- **Tests**: integration `crates/gitty-core/tests/reconcile.rs` (move repo; two clones same fingerprint; empty repo).
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-04, DISC-09, DISC-10

## T8 — Public API surface in `lib.rs`

- **What**: Re-export config/repository/scan/reconcile/git read + a thin `Workspace::scan_and_reconcile(path)` convenience used by CLI.
- **Where**: `crates/gitty-core/src/lib.rs`.
- **Depends on**: T3, T7
- **Done when**: a single call from outside the crate can scan + reconcile + return a report.
- **Gate**: `cargo test -p gitty-core`
- **Covers**: DISC-01..10 integration

## T9 — CLI `scan` / `list` / `status`

- **What**: clap subcommands; load/save Config; print summaries; empty-state messages; per-repo status table with errored-repo flagging.
- **Where**: `crates/gitty-cli/src/main.rs` (+ helpers).
- **Depends on**: T8
- **Done when**: `gitty scan <tmp>`, `gitty list`, `gitty status` behave per spec on a real temp tree.
- **Tests**: `crates/gitty-cli/tests/cli.rs` via `assert_cmd` (scan a temp tree, list shows repos, status shows branch/dirty). Use a `GITTY_CONFIG_DIR` env override so tests don't touch the real config.
- **Gate**: `cargo test`
- **Covers**: DISC-06, DISC-07, DISC-08

## T10 — Final gate & cleanup

- **What**: Remove scaffold `add()`/`Version`-only CLI remnants as needed; ensure no clippy warnings; fmt.
- **Depends on**: T9
- **Done when**: full gate green.
- **Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check && npm run check`
- **Covers**: Success Criteria

## Traceability (tasks ↔ requirements)

| Req | Task(s) |
| --- | --- |
| DISC-01 | T6, T8 |
| DISC-02 | T4, T5 |
| DISC-03 | T6 |
| DISC-04 | T7 |
| DISC-05 | T3 |
| DISC-06 | T9 |
| DISC-07 | T5, T9 |
| DISC-08 | T9 |
| DISC-09 | T7 |
| DISC-10 | T7 |
| DISC-11 | T5 |
| DISC-12 | T5 |

**Coverage:** 12/12 requirements mapped.

## Suggested commit sequence (atomic)

1. `chore: add foundation deps (git2, dirs, walkdir, thiserror, anyhow)` — T1
2. `feat(core): error types and config persistence` — T2, T3
3. `feat(core): repository model and registry` — T4
4. `feat(core): git2 read layer (status, fingerprint, head, changes)` — T5
5. `feat(core): scan root discovery` — T6
6. `feat(core): reconcile with collision-safe re-linking` — T7, T8
7. `feat(cli): scan, list, status commands` — T9
8. `chore: clippy/fmt cleanup` — T10
