# Foundation — Repository Discovery & Registry Specification

> Slice 1 of Milestone 1 (Foundation). Slices 2 (Git write layer) and 3 (Lock mechanism)
> follow. Decisions locked in ADR-0004/0005/0006 and STATE.md D5–D12.

## Problem Statement

Developers managing many local Git repositories have no single place to discover, track, and inspect them. Before Gitty can run bulk operations or show a health dashboard, it must reliably **find** repositories under user-chosen directories, **remember** them with a stable identity that survives moves, and **report** each one's current Git state. This slice delivers that foundation — headless, through the CLI.

## Goals

- [ ] Discover all standard Git repositories under one or more Scan Roots in <5s for 50+ repos
- [ ] Persist a Repository registry with stable UUID identity that survives filesystem moves
- [ ] Report per-Repository status (branch, dirty, ahead/behind, HEAD commit, changed files) from git2
- [ ] Expose it all through `gitty scan`, `gitty list`, `gitty status`

## Out of Scope

| Feature | Reason |
| --- | --- |
| Git write operations (fetch/pull/checkout) | Slice 2 |
| Lock mechanism | Slice 3 |
| Macros / Steps / Jobs | Milestone 4 — Foundation builds primitives only (D5/A2) |
| Groups / Tags assignment & filtering | Milestone 4 (fields pre-modelled only, per C3) |
| Bare repositories, worktrees | Out of v1 (D8) |
| Submodule-aware handling (flag/filter) | Treated as generic nested repos in slice 1 (D8) |
| Manual re-link resolution for ambiguous matches | Deferred (ADR-0005) |
| Time-windowed Change Dashboard | Milestone 5 |
| Following symlinks during scan | Excluded to avoid cycles (D8) |
| Config schema migrations | None during 0.x; hard-error on mismatch (ADR-0004) |
| Any UI | Milestone 3 |

---

## User Stories

### P1: Discover & register Repositories under a Scan Root ⭐ MVP

**User Story**: As a developer, I want to point Gitty at a directory and have it find every Git repository underneath, so that I no longer track them by hand.

**Why P1**: Discovery + persistence is the bedrock every later feature builds on.

**Acceptance Criteria**:

1. WHEN the user runs `gitty scan <path>` on a directory THEN the system SHALL recursively walk it, register every directory containing a `.git` entry as a Repository, and persist them to the Config file.
2. WHEN a Repository is first registered THEN the system SHALL assign it a UUID (v4), record its absolute path, and record its content fingerprint (root-commit OID, or null if none).
3. WHEN the scan encounters a `.git` directory THEN the system SHALL record the repo but SHALL NOT descend into the `.git` directory itself.
4. WHEN the scan encounters a directory in the default ignore list (`node_modules`, `target`, `.venv`, `dist`, `build`, `.next`) THEN the system SHALL skip descending into it.
5. WHEN the scan encounters a symlink THEN the system SHALL NOT follow it.
6. WHEN the same `gitty scan <path>` is run again on an unchanged tree THEN the registry SHALL be idempotent (no duplicate Repositories created).
7. WHEN no Config file exists THEN the system SHALL create one (with schema `version`) at the platform config path on first write.

**Independent Test**: Create a temp tree with 3 nested git repos + a `node_modules` decoy; run `gitty scan`; assert 3 Repositories persisted with UUIDs and paths.

---

### P1: List Repositories and inspect status ⭐ MVP

**User Story**: As a developer, I want to list my tracked repositories and see each one's Git status, so that I can scan the health of my workspace at a glance.

**Why P1**: Discovery is only useful if the results are readable. Completes the vertical slice.

**Acceptance Criteria**:

1. WHEN the user runs `gitty list` THEN the system SHALL print every registered Repository with its path and state (active or missing).
2. WHEN the user runs `gitty status` THEN the system SHALL print, for each active Repository, its current branch, detached-HEAD flag, dirty flag, and ahead/behind counts vs. upstream.
3. WHEN a Repository has no upstream configured THEN the system SHALL report ahead/behind as not-applicable rather than failing.
4. WHEN the registry is empty THEN `gitty list` / `gitty status` SHALL print a friendly empty-state message, not an error.

**Independent Test**: Scan a temp repo on branch `main` with an uncommitted file; run `gitty status`; assert output shows `main` and dirty=true.

---

### P2: Re-linking & Missing state on rescan

**User Story**: As a developer who reorganises folders, I want Gitty to keep a repository's identity when I move it, so that my organization and history aren't lost.

**Why P2**: Important for trust, but the MVP demos without a move occurring.

**Acceptance Criteria**:

1. WHEN a rescan finds that a registered Repository's recorded path no longer exists THEN the system SHALL mark it `missing` and SHALL NOT delete it.
2. WHEN exactly one `missing` Repository and exactly one newly-discovered repository share a single non-null content fingerprint THEN the system SHALL re-link: update the path, preserve the UUID, and clear the `missing` state.
3. WHEN a fingerprint is shared by more than one candidate (clone/fork collision) THEN the system SHALL NOT auto-relink; the `missing` Repository stays `missing` and the new one is registered fresh.
4. WHEN a discovered repository has no commits (null fingerprint) THEN the system SHALL register it fresh and SHALL NOT use it for re-linking.

**Independent Test**: Register a repo, move its directory, rescan; assert the same UUID now points at the new path and state is active.

---

### P2: Rich per-Repository status (HEAD commit + changed files)

**User Story**: As a developer, I want to see each repository's latest commit and what's changed, so that I have the data a dashboard needs.

**Why P2**: Builds on P1 status; enables the future dashboard but isn't required to demo discovery.

**Acceptance Criteria**:

1. WHEN status is computed for a Repository with at least one commit THEN the system SHALL include the HEAD commit's short hash, author name, commit date, and subject line.
2. WHEN a Repository has working-tree changes THEN the system SHALL include a list of changed files, each with a status (added, modified, deleted, renamed, untracked).
3. WHEN a Repository has no commits THEN the system SHALL report an empty HEAD summary without failing.

**Independent Test**: In a temp repo, commit one file then modify another; assert status includes the commit subject and the modified file with status=modified.

---

## Edge Cases

- WHEN `gitty scan <path>` targets a non-existent path THEN the system SHALL return a clear error.
- WHEN a Scan Root contains no Git repositories THEN the scan SHALL succeed and register zero Repositories.
- WHEN the Config file is corrupt/unparseable THEN the system SHALL fail with a clear error and not overwrite it.
- WHEN the Config schema `version` is unrecognised THEN the system SHALL hard-error (no migration) per ADR-0004.
- WHEN libgit2 cannot open a discovered repo (corrupt `.git`) THEN status SHALL mark that Repository as errored and continue with the rest.
- WHEN two Scan Roots overlap and discover the same path THEN the Repository SHALL be registered once (dedupe by canonical path).
- WHEN a repository directory is read-permission-denied THEN the scan SHALL skip it and continue.

---

## Requirement Traceability

| Requirement ID | Story | Phase | Status |
| --- | --- | --- | --- |
| DISC-01 | P1: Discover | Execute | Verified |
| DISC-02 | P1: Discover (UUID + fingerprint) | Execute | Verified |
| DISC-03 | P1: Discover (skip `.git`/ignore/symlink) | Execute | Verified |
| DISC-04 | P1: Discover (idempotent rescan) | Execute | Verified |
| DISC-05 | P1: Config create/load/save + schema version | Execute | Verified |
| DISC-06 | P1: List Repositories | Execute | Verified |
| DISC-07 | P1: Status (branch/detached/dirty/ahead-behind) | Execute | Verified |
| DISC-08 | P1: Empty-state handling | Execute | Verified |
| DISC-09 | P2: Missing state on rescan | Execute | Verified |
| DISC-10 | P2: Collision-safe re-link | Execute | Verified |
| DISC-11 | P2: HEAD commit summary | Execute | Verified |
| DISC-12 | P2: Changed-files list | Execute | Verified |

**ID format:** `DISC-NN`
**Status values:** Pending → In Design → In Tasks → Implementing → Verified
**Coverage:** 12 total, 12 mapped to tasks, 12 verified by passing tests + CLI smoke test

---

## Success Criteria

- [ ] `gitty scan`, `gitty list`, `gitty status` all work end-to-end against real local repos
- [ ] Scanning a 50+ repo tree completes in <5 seconds
- [ ] Moving a repo and rescanning preserves its UUID (unambiguous case)
- [ ] `cargo test` passes for `gitty-core` and `gitty-cli`; `cargo clippy -- -D warnings` clean
- [ ] Config round-trips (write then read) without data loss
