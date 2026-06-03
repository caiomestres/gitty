# Gitty

Synchronization, orchestration, and workspace health platform for developers managing large collections of Git repositories.

Gitty helps developers avoid operating on stale code by providing centralized repository management, bulk operations, intelligent scheduling, and workspace health monitoring.

## Features

- **Repository Discovery** — Point at root folders, automatically find all git repos. Stable UUID identity survives moves via content-fingerprint re-linking.
- **Groups & Tags** — Hierarchical group tree (arbitrary nesting) + additive tags for cross-cutting filters. Built-in Favorite tag.
- **Unified Macro Engine** — Named command sequences (Git operations + shell commands) with variables, conditions, rollback, and confirmations.
- **Workspace Health** — Per-repo traffic light (healthy / warning / critical) based on freshness, divergence, dirty tree, and detached HEAD. Aggregate score across the workspace.
- **Change Dashboard** — What changed across your workspace: commits grouped by author, repository, or branch over configurable time windows (24h, 7d, 30d).
- **Intelligent Scheduler** — Background fetch/sync with time-of-day, day-of-week, and power-source conditions. Battery-aware (pauses on low battery).
- **Notifications** — Configurable health alerts (critical-only, any change, scheduler complete) with in-app panel and OS-native toast support.
- **GUI + CLI** — Use either independently; shared config file with file-level locking.

## Architecture

```
gitty/
├── crates/
│   ├── gitty-core/       # Domain logic (no framework deps)
│   └── gitty-cli/        # Standalone Rust CLI (clap)
├── src-tauri/            # Tauri 2 desktop app (thin shell over core)
├── src/                  # SvelteKit 5 frontend (SPA)
│   ├── lib/
│   │   ├── components/   # AppShell, Sidebar, NotificationPanel
│   │   └── types/        # TypeScript DTOs for IPC
│   └── routes/           # Pages: dashboard, health, changes, settings
├── docs/
│   └── adr/              # Architecture Decision Records
├── CONTEXT.md            # Domain glossary
├── DESIGN.md             # UI design system
└── AGENTS.md             # Agent rules & skill catalog
```

## Tech Stack

| Layer | Tech | Notes |
|-------|------|-------|
| Desktop | Tauri 2 | IPC, native APIs, security |
| Frontend | SvelteKit 5 + TypeScript | Static adapter, Svelte 5 runes |
| Backend | Rust (2021 edition) | Cargo workspace, three crates |
| Git (reads) | git2 (libgit2, vendored) | Status, log, branch info, revwalk |
| Git (writes) | System `git` CLI | Full compatibility with user config/hooks |
| Scheduling | battery crate + time crate | Power-aware, time-windowed triggers |
| Tests (BE) | cargo test | 165 tests (unit + integration across 3 crates) |
| Tests (FE) | Vitest (planned) | Unit + integration |

## Platforms

- Windows
- macOS
- Linux (Ubuntu)

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (LTS)
- [Git](https://git-scm.com/)
- Platform-specific Tauri dependencies ([see Tauri docs](https://v2.tauri.app/start/prerequisites/))

### Development

```bash
# Install frontend dependencies
npm install

# Run in development mode (frontend + backend)
npm run tauri dev

# Type checking
npm run check

# Rust tests (all crates)
cargo test

# Rust linting
cargo clippy -- -D warnings

# Rust formatting
cargo fmt
```

### Build

```bash
# Production build
npm run tauri build
```

## Configuration

Gitty stores its config at the platform-native location:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\gitty\config.json` |
| macOS | `~/Library/Application Support/gitty/config.json` |
| Linux | `~/.config/gitty/config.json` |

Health cache is stored alongside the config as `health.json`. The scheduler daemon writes a `scheduler.pid` file in the same directory.

## CLI Usage

```bash
gitty scan <path>                       # Discover repos in a directory
gitty list                              # List discovered repos
gitty status                            # Show branch/dirty/tracking for all repos
gitty fetch [--repo <name>]             # Fetch all remotes
gitty pull [--repo <name>]              # Pull all repos
gitty checkout <branch> [--repo <name>] # Checkout a branch

gitty group list                        # List all groups
gitty group create <name> [--parent X]  # Create a group
gitty group assign <repo> <group>       # Assign repo to group
gitty group tree                        # Show group hierarchy

gitty tag list                          # List all tags in use
gitty tag add <repo> <tag>              # Tag a repository
gitty filter --group X --tag Y          # Filter repos by group/tag

gitty macro list                        # List defined macros
gitty macro define <name> <steps...>    # Define a macro (fetch, pull, checkout:branch, shell:cmd)
gitty macro run <name> [--group X]      # Run a macro against a selection

gitty health                            # Show workspace health score
gitty health --repo <name>              # Show detailed health for one repo

gitty scheduler start                   # Start background scheduler daemon
gitty scheduler stop                    # Stop the scheduler
gitty scheduler status                  # Show scheduler status

gitty notification show                 # Show notification trigger config
gitty notification set <mode>           # Set trigger: on-critical, on-any-change, on-scheduler-complete, disabled
```

## Domain Language

Gitty uses precise terminology defined in [`CONTEXT.md`](./CONTEXT.md):

| Term | Meaning |
|------|---------|
| **Workspace** | Named collection of Scan Roots managed as a unit |
| **Scan Root** | Directory scanned recursively for `.git` dirs |
| **Repository** | A discovered git repo, identified by UUID |
| **Group** | Hierarchical category (tree structure, one per repo) |
| **Tag** | Additive label (many per repo; Favorite is built-in) |
| **Macro** | Named sequence of Steps targeting a repo selection |
| **Step** | Git Operation (fetch/pull/checkout) or Shell Command |
| **Job** | One Macro execution on one Repository |
| **Health Check** | Evaluation of a repo against a specific criterion |
| **Scheduler** | Background automation engine for Macros |
| **Notification** | Timestamped alert for health changes |

## Current Status

**Milestones 1-5 complete.** Core infrastructure, CLI, desktop GUI, organization (groups/tags/macros), health checks, change dashboard, scheduler, and notifications are all implemented.

**Milestone 6 (Polish & Ship)** is next: full design system compliance, error handling refinements, packaging, code signing, and user documentation.

## License

MIT
