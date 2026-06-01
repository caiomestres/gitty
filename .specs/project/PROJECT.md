# Gitty

**Vision:** A desktop Git client and CLI that lets developers synchronize, monitor, and operate on large collections of Git repositories from a single dashboard — replacing scattered terminal tabs and manual `cd`-and-pull loops with structured, health-aware workspace management.

**For:** Developers managing 10–100+ local Git repositories (microservice teams, polyrepo orgs, open-source contributors).

**Solves:** The daily friction of keeping many repos up to date, spotting drift before it causes merge pain, and running bulk operations without writing shell scripts.

## Goals

- Provide a single Workspace view showing health status across all managed Repositories (measured by: user can scan 50+ repos in <5 seconds)
- Enable bulk Git operations via Macros that target any slice of Repositories (measured by: fetch/pull/checkout across N repos with one command)
- Deliver a standalone CLI that works identically to the GUI for headless/CI workflows (measured by: CLI and GUI share 100% of core logic via `gitty-core`)
- Ship a polished, Cursor-inspired desktop UI that feels editorial, not cluttered (measured by: adherence to DESIGN.md system)

## Tech Stack

**Core:**

- Desktop framework: Tauri 2
- Frontend: SvelteKit 5 + TypeScript (static adapter, SPA mode)
- Backend: Rust (2021 edition)
- Git reads: git2 (libgit2 bindings)
- Git writes: system `git` CLI (shell-out)

**Key dependencies:**

- `@tauri-apps/api` v2 — IPC bridge
- `serde` / `serde_json` — Rust serialization
- `tauri-plugin-opener` — native file/URL opener
- `clap` — CLI argument parsing (planned, not yet added)
- `git2` — Git read operations (planned, not yet added)

## Scope

**v1 includes:**

- Workspace with Scan Roots — discover Repositories by scanning directories
- Repository dashboard — health status, branch info, dirty state
- Change Dashboard — what changed across workspace (commits, authors, time windows)
- Groups and Tags — organize Repositories (arbitrary nesting for Groups)
- Macros — fetch, pull, checkout, custom shell commands across selections; variables, conditions, rollback, confirmations
- Scheduler — background fetch on configurable triggers
- Health Checks — freshness, divergence, dirty tree, detached HEAD
- Standalone CLI — same operations, headless
- Config persistence — file-based, shared between CLI and GUI
- Lock — prevent concurrent operations on the same Repository

**Explicitly out of scope:**

- Dependency Map (deferred to v2)
- Remote hosting integration (GitHub/GitLab API)
- Code review or PR management
- Built-in merge conflict resolution
- Multi-user / team features
- Mobile support

## Constraints

- Technical: Users must have `git` installed for write operations (ADR-0001)
- Architecture: Cargo workspace with 3 crates — `gitty-core`, `gitty-cli`, `gitty-tauri` (ADR-0002)
- Language: All code, comments, and documentation in English (ADR-0003)
- Design: UI must follow DESIGN.md system (warm cream canvas, Cursor Orange scarcely, hairline depth)
