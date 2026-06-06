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
- Git reads: git2 (libgit2 bindings, vendored)
- Git writes: system `git` CLI (shell-out)

**Key dependencies:**

- `@tauri-apps/api` v2 — IPC bridge
- `serde` / `serde_json` — Rust serialization
- `tauri-plugin-opener` — native file/URL opener
- `clap` 4.x — CLI argument parsing (derive mode)
- `git2` 0.21 — Git read operations (vendored libgit2)
- `battery` 0.7 — Power-state aware scheduling
- `notify` 7.x — File watching for config changes
- `rayon` 1.x — Parallel iteration

## Scope

**v1 includes:**

- Workspace with Scan Roots — discover Repositories by scanning directories
- Repository dashboard — health status, branch info, dirty state
- Change Dashboard — what changed across workspace (commits, authors, time windows)
- Groups and Tags — organize Repositories (arbitrary nesting for Groups)
- Macros — fetch, pull, checkout, custom shell commands across selections; variables, conditions, rollback, retry (network errors only)
- Scheduler — background Macro execution on configurable triggers (simple/advanced, power-aware)
- Health Checks — freshness, divergence, dirty tree, detached HEAD (trait-based, extensible)
- Notifications — OS-native toasts + in-app panel for critical health warnings
- Standalone CLI — same operations, headless, with daemon scheduler
- Config persistence — file-based JSON, shared between CLI and GUI (file-watcher for cross-process sync)
- Lock — prevent concurrent operations on the same Repository (PID-based, stale detection)
- Distribution — GitHub Actions release (Windows NSIS, macOS DMG, Linux AppImage), Homebrew tap
- Documentation — MkDocs Material site (Getting Started, Concepts, CLI Reference, GUI Guide)

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
- Distribution: Windows code signing via SignPath.io, macOS ad-hoc codesign (no Apple Developer account for v1, ADR-0009)
