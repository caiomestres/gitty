# Gitty

Synchronization, orchestration, and workspace health platform for developers managing large collections of Git repositories.

Gitty helps developers and AI tools avoid operating on stale code by providing centralized repository management, bulk operations, intelligent scheduling, and workspace health monitoring.

## Features

- **Repository Discovery** — Point at root folders, automatically find all git repos
- **Unified Macro Engine** — Named command sequences (single-step or multi-step) targeting any selection of repos
- **Parallel Execution** — Concurrent job engine with configurable workers, retry logic, and error handling
- **Workspace Health** — Per-repo traffic light (healthy / warning / critical) with aggregate scoring
- **Intelligent Scheduler** — Background fetch/sync with time and power-source conditions
- **Change Dashboard** — What changed across your workspace: commits, authors, repos, time windows
- **GUI + CLI** — Use either independently; shared config with file-level locking

## Architecture

```
gitty/
├── crates/
│   ├── gitty-core/       # Domain logic (no framework deps)
│   ├── gitty-cli/        # Standalone Rust CLI (clap)
│   └── gitty-tauri/      # Tauri 2 desktop app
├── src/                  # SvelteKit 5 frontend
├── docs/
│   ├── adr/              # Architecture Decision Records
│   └── agents/           # Agent skill config
├── CONTEXT.md            # Domain glossary
├── DESIGN.md             # UI design system
└── AGENTS.md             # Agent rules & skill catalog
```

## Tech Stack

| Layer | Tech | Notes |
|-------|------|-------|
| Desktop | Tauri 2 | IPC, native APIs, security |
| Frontend | SvelteKit 5 + TypeScript | Static adapter, Vite |
| Backend | Rust | Cargo workspace, three crates |
| Git (reads) | git2 (libgit2) | Status, log, branch info |
| Git (writes) | System `git` CLI | Full compatibility with user config/hooks |
| Tests (FE) | Vitest | Unit + integration |
| Tests (BE) | cargo test | Unit + integration |

## Platforms

- Windows
- Ubuntu (Linux)
- macOS

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

# Rust tests
cd src-tauri && cargo test

# Rust linting
cd src-tauri && cargo clippy -- -D warnings
```

## Configuration

Gitty stores its config at the platform-native location:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\gitty\config.json` |
| macOS | `~/Library/Application Support/gitty/config.json` |
| Linux | `~/.config/gitty/config.json` |

## CLI Usage

```bash
gitty init                          # Create config, add first scan root
gitty scan                          # Discover repos in scan roots
gitty list                          # List discovered repos
gitty pull [--group X] [--tag Y]    # Bulk pull
gitty fetch [--group X] [--tag Y]   # Bulk fetch
gitty run "command" [--group X]     # Arbitrary command on repos
gitty macro run "Safe Pull"         # Run named macro
gitty macro list                    # List macros
gitty health                        # Show workspace health
gitty status                        # Quick overview
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
| **Job** | One Macro execution on one Repository |
| **Health Check** | Evaluation of a repo against a specific criterion |

## License

MIT
