# Foundation — Discovery & Registry Design

> Implements `spec.md`. Architecture decisions already locked (ADR-0001/0004/0005/0006).
> This doc fixes the module layout, data shapes, and key algorithms before coding.

## Crate & module layout (`gitty-core`)

```
crates/gitty-core/src/
├── lib.rs            # re-exports public API
├── error.rs          # CoreError (thiserror) + Result alias
├── config/
│   ├── mod.rs        # Config struct, schema version, load/save, path resolution (dirs)
│   └── paths.rs      # config_dir / config_file / locks_dir resolution
├── repository.rs     # Repository, RepositoryState, ScanRoot, Registry
├── scan.rs           # walkdir-based discovery -> Vec<DiscoveredRepo>
├── reconcile.rs      # merge discovered repos into Registry (re-link / missing)
└── git/
    ├── mod.rs
    └── read.rs       # git2-backed status, fingerprint, HEAD summary, changed files
```

The `foundation-git-write` and `foundation-lock` features add `git/write.rs` and `lock.rs` later — not here.

## Data structures

```rust
// repository.rs
pub struct Config {
    pub version: u32,             // schema version (ADR-0004); current = 1
    pub workspace: Workspace,     // single implicit default workspace (CONTEXT.md)
}

pub struct Workspace {
    pub scan_roots: Vec<ScanRoot>,
    pub repositories: Vec<Repository>,
}

pub struct ScanRoot { pub path: PathBuf }

pub struct Repository {
    pub id: Uuid,
    pub path: PathBuf,                 // canonicalized absolute path
    pub fingerprint: Option<String>,  // root-commit OID hex; None if no commits
    pub state: RepositoryState,
    // pre-modelled for M4 (C3) — defaulted now, unused in this feature
    pub group_id: Option<Uuid>,
    pub tags: Vec<String>,
}

pub enum RepositoryState { Active, Missing }
```

```rust
// git/read.rs
pub struct RepositoryStatus {
    pub branch: Option<String>,     // None when detached or unborn
    pub detached: bool,
    pub dirty: bool,
    pub upstream: Option<Upstream>, // None when no tracking branch
    pub head: Option<CommitSummary>,
    pub changed_files: Vec<ChangedFile>,
}
pub struct Upstream { pub ahead: usize, pub behind: usize }
pub struct CommitSummary { pub short_id: String, pub author: String, pub date: String, pub subject: String }
pub struct ChangedFile { pub path: String, pub status: ChangeStatus }
pub enum ChangeStatus { Added, Modified, Deleted, Renamed, Untracked }
```

```rust
// scan.rs
pub struct DiscoveredRepo { pub path: PathBuf, pub fingerprint: Option<String> }
```

## Key algorithms

### Scan (`scan.rs`) — DISC-01/03

- `walkdir::WalkDir::new(root).follow_links(false)`.
- For each directory, test for a child `.git` (dir or file). If present → it's a repo:
  emit `DiscoveredRepo`, and **prune** further descent into that repo's tree is *not* done
  (D8: descend into nested repos) — but always skip the `.git` entry itself.
- `filter_entry` prunes: any directory named in the ignore list, and `.git` directories.
- Fingerprint is computed lazily by the git layer, not during the walk, to keep the walk fast.
- Dedupe discovered paths by canonicalized path (overlapping roots edge case).

> Performance note (<5s/50 repos goal): pruning the ignore list + never recursing into `.git`
> keeps the file count bounded. Fingerprinting opens each repo once via git2.

### Reconcile (`reconcile.rs`) — DISC-04/09/10

Given the existing `Registry` and the freshly `DiscoveredRepo`s for a scan root:

1. Index existing repos by canonical path.
2. For each discovered repo:
   - If a registered repo has the same path → keep it (idempotent), refresh fingerprint, mark Active.
   - Else it's a candidate "new" repo (hold for step 4).
3. Mark every registered repo whose path is absent from disk as `Missing`.
4. **Collision-safe re-link**: build a multimap `fingerprint -> [missing repos]` and
   `fingerprint -> [new repos]`. For each non-null fingerprint where **both** lists have
   length exactly 1 → re-link (move UUID to new path, state=Active, drop the new candidate).
   All other new candidates → register fresh with a new UUID. Null fingerprints never relink.

### Git read (`git/read.rs`) — DISC-02/07/11/12

- `git2::Repository::open(path)`.
- `fingerprint`: walk to the root commit (revwalk from HEAD, `set_sorting(TOPOLOGICAL|REVERSE)`,
  take first) → its OID hex. Unborn HEAD → `None`.
- `branch`: `repo.head()`; if `head_detached()` → `detached=true, branch=None`.
- `dirty`: `repo.statuses()` with untracked included; non-empty (excluding ignored) → dirty.
- `upstream`: resolve tracking branch via `branch.upstream()`; `graph_ahead_behind`.
- `head` summary: HEAD commit → short id, author name, time (RFC3339), summary line.
- `changed_files`: map `Status` flags to `ChangeStatus`.
- Open/read failure → surfaced as `CoreError::Git`; caller (status command) marks that repo
  errored and continues (edge case).

### Config (`config/`) — DISC-05

- Path: `dirs::config_dir()/gitty/config.json`; locks at `.../gitty/locks/`.
- `Config::load()`: if file missing → return default in-memory config (don't create yet);
  if present → parse, check `version == CURRENT_SCHEMA_VERSION` else `CoreError::UnsupportedSchema`.
- `Config::save()`: create parent dir, write pretty JSON atomically (temp file + rename).

## CLI surface (`gitty-cli`) — DISC-06/08

| Command | Behavior |
| --- | --- |
| `gitty scan <path>` | Add `<path>` as a Scan Root (if new), discover, reconcile, save Config. Print summary (N found, M new, K re-linked, J missing). |
| `gitty list` | Load Config, print each Repository: path + state. Empty → friendly message. |
| `gitty status` | Load Config, compute status per Active repo via git read, print table. Errored repos flagged. |

CLI uses `anyhow` at the boundary; maps `CoreError` to user-facing messages.

## Test strategy (per TESTING.md)

- **gitty-core unit/integration**: temp dirs + real `git2`-created repos (init, commit, branch).
  Tests live in `crates/gitty-core/tests/` (integration) and inline `#[cfg(test)]` (unit).
- Helper: a small test fixture that inits a repo, commits, optionally adds a remote/upstream.
- **gitty-cli integration**: invoke the binary with `assert_cmd` (add as dev-dep) against temp trees.
- Gate: `cargo test` + `cargo clippy -- -D warnings`.

## New dependencies

| Crate | Where | Why |
| --- | --- | --- |
| `dirs` | gitty-core | config path resolution (D6/B4) |
| `walkdir` | gitty-core | directory traversal (D8/D4) |
| `git2` | gitty-core | read layer (ADR-0001); vendored libgit2 |
| `thiserror` | gitty-core | typed errors (D12) |
| `anyhow` | gitty-cli | binary-boundary errors (D12) |
| `assert_cmd`, `tempfile` | dev-deps | CLI + fs tests |
