# Foundation — Git Write Layer Specification

> Second feature of Milestone 1 (Foundation). Follows `foundation-discovery`; the
> `foundation-lock` feature follows this one. Decisions locked in ADR-0001 and STATE.md D10/D12.

## Problem Statement

Gitty can discover, track, and read the status of Git repositories (via `foundation-discovery`), but it cannot act on them. Before Macros, bulk operations, or even a simple "fetch all", Gitty needs a write layer that shells out to the system `git` CLI. This feature delivers a typed, safe shell-out runner for git write operations — fetch, pull, and checkout — exposed through the CLI.

## Goals

- [ ] Locate and validate the system `git` binary at startup
- [ ] Execute git write operations (fetch, pull, checkout) on any registered Repository via shell-out
- [ ] Return structured results — exit code, stdout, stderr — for every operation
- [ ] Classify git exit codes into actionable error categories (network, conflict, auth, unknown)
- [ ] Expose `gitty fetch`, `gitty pull`, `gitty checkout` CLI commands targeting individual or all repositories

## Out of Scope

| Feature | Reason |
| --- | --- |
| Lock mechanism | `foundation-lock` feature |
| Retry logic for transient/network errors | Milestone 4 (Macro scripting) — the runner reports the error category but does not retry |
| Macros / Steps / Jobs | Milestone 4 — this feature builds the primitive runner only |
| Progress streaming / real-time output | Milestone 3 (desktop shell) — CLI captures output after completion |
| Rebase, merge, stash, clean, reset | Added incrementally as needed; fetch/pull/checkout cover the Foundation use case |
| Remote credential management | Relies on user's existing git credential helpers |
| Any UI | Milestone 3 |

---

## User Stories

### P1: Execute git fetch on registered Repositories ⭐ MVP

**User Story**: As a developer, I want to run `gitty fetch` to fetch all my tracked repositories at once, so that I can keep them up to date without visiting each one manually.

**Why P1**: Fetch is the safest write operation (no working-tree changes) and the default Scheduler action. It validates the entire shell-out pipeline end-to-end.

**Acceptance Criteria**:

1. WHEN the user runs `gitty fetch` THEN the system SHALL execute `git fetch --all` in each active Repository and report success or failure per repo.
2. WHEN the user runs `gitty fetch <path-or-name>` THEN the system SHALL execute fetch only on the matching Repository.
3. WHEN a fetch fails (non-zero exit code) THEN the system SHALL report the Repository path, exit code, and stderr, and SHALL continue with the remaining repositories.
4. WHEN the system `git` binary is not found on PATH THEN the system SHALL fail with a clear error before attempting any operation.
5. WHEN the system `git` binary is found THEN the system SHALL validate it by running `git --version` and extracting the version string.

**Independent Test**: Register a temp repo with a remote; run `gitty fetch`; assert exit code 0 and output indicates fetch succeeded.

---

### P1: Execute git pull on registered Repositories ⭐ MVP

**User Story**: As a developer, I want to run `gitty pull` to pull all my tracked repositories, so that I can bring them all up to date in one command.

**Why P1**: Pull is the most common bulk operation. Together with fetch, it validates the runner handles both safe and working-tree-modifying operations.

**Acceptance Criteria**:

1. WHEN the user runs `gitty pull` THEN the system SHALL execute `git pull` in each active Repository and report success or failure per repo.
2. WHEN the user runs `gitty pull <path-or-name>` THEN the system SHALL execute pull only on the matching Repository.
3. WHEN a pull fails (e.g., merge conflict, no upstream) THEN the system SHALL report the error category (conflict, no-upstream, network, auth, unknown), the exit code, and stderr.
4. WHEN a Repository has no upstream configured THEN the pull SHALL fail gracefully and the system SHALL report it as a no-upstream error, not crash.

**Independent Test**: Register a temp repo with a local remote, push a commit to the remote; run `gitty pull`; assert the working tree is updated.

---

### P2: Execute git checkout on registered Repositories

**User Story**: As a developer, I want to run `gitty checkout <branch>` across my repositories, so that I can switch a group of repos to the same branch for a cross-repo feature.

**Why P2**: Checkout is important but less frequently bulk-applied than fetch/pull.

**Acceptance Criteria**:

1. WHEN the user runs `gitty checkout <branch>` THEN the system SHALL execute `git checkout <branch>` in each active Repository and report success or failure per repo.
2. WHEN the user runs `gitty checkout <branch> --repo <path-or-name>` THEN the system SHALL execute checkout only on the matching Repository.
3. WHEN a checkout fails (e.g., branch not found, dirty working tree) THEN the system SHALL report the error with the Repository path and stderr.
4. WHEN a Repository does not have the requested branch THEN the system SHALL report it as a branch-not-found error and continue with other repos.

**Independent Test**: Register a temp repo with branches `main` and `dev`; run `gitty checkout dev`; assert HEAD now points to `dev`.

---

### P1: Git binary discovery and validation ⭐ MVP

**User Story**: As a developer, I want Gitty to tell me clearly if git isn't installed, so that I don't get cryptic errors when trying to fetch or pull.

**Why P1**: Prerequisite for every write operation.

**Acceptance Criteria**:

1. WHEN a git write command is invoked THEN the system SHALL first locate the `git` binary via PATH.
2. WHEN `git` is found THEN the system SHALL execute `git --version` and parse the version string for validation.
3. WHEN `git` is not found on PATH THEN the system SHALL return a `GitNotFound` error with a message instructing the user to install Git.
4. WHEN the `git --version` output cannot be parsed THEN the system SHALL warn but proceed (non-blocking — some git distributions have unusual version strings).

**Independent Test**: Mock or override PATH to exclude git; assert `GitNotFound` error is returned.

---

## Edge Cases

- WHEN a Repository path no longer exists on disk (state=Missing) THEN the write command SHALL skip it and report it as skipped.
- WHEN the user's git config requires interactive input (e.g., SSH passphrase, credential prompt) THEN the operation SHALL time out or fail; Gitty does not relay interactive prompts in v1.
- WHEN `git fetch` or `git pull` encounters a network error THEN the system SHALL classify it as a `Network` error category.
- WHEN `git pull` encounters a merge conflict THEN the system SHALL classify it as a `Conflict` error category.
- WHEN `git checkout` fails due to uncommitted changes THEN the system SHALL classify it as a `DirtyWorkTree` error category.
- WHEN `git` produces output on stderr but exits with code 0 (informational warnings) THEN the system SHALL treat it as success.
- WHEN multiple repositories are targeted and some succeed while others fail THEN the system SHALL complete all attempts and report per-repo results, not abort on first failure.

---

## Requirement Traceability

| Requirement ID | Story | Phase | Status |
| --- | --- | --- | --- |
| GWRITE-01 | P1: Git binary discovery and validation | Pending | Pending |
| GWRITE-02 | P1: Shell-out runner — execute git commands, capture output | Pending | Pending |
| GWRITE-03 | P1: Exit code classification (network, conflict, auth, unknown) | Pending | Pending |
| GWRITE-04 | P1: Fetch — `git fetch --all` on one or all repos | Pending | Pending |
| GWRITE-05 | P1: Pull — `git pull` on one or all repos | Pending | Pending |
| GWRITE-06 | P2: Checkout — `git checkout <branch>` on one or all repos | Pending | Pending |
| GWRITE-07 | P1: CLI `gitty fetch` command | Pending | Pending |
| GWRITE-08 | P1: CLI `gitty pull` command | Pending | Pending |
| GWRITE-09 | P2: CLI `gitty checkout` command | Pending | Pending |
| GWRITE-10 | P1: Per-repo result reporting (success/failure, continue on error) | Pending | Pending |
| GWRITE-11 | P1: Skip Missing repositories | Pending | Pending |

**ID format:** `GWRITE-NN`
**Status values:** Pending → In Design → In Tasks → Implementing → Verified
**Coverage:** 11 total

---

## Success Criteria

- [ ] `gitty fetch`, `gitty pull`, `gitty checkout` all work end-to-end against real local repos with remotes
- [ ] Missing repos are skipped with a clear message
- [ ] Operations that fail on some repos continue on the rest and report per-repo results
- [ ] `git` not on PATH produces a clear, actionable error
- [ ] Exit codes are classified into meaningful error categories
- [ ] `cargo test` passes for `gitty-core` and `gitty-cli`; `cargo clippy -- -D warnings` clean
