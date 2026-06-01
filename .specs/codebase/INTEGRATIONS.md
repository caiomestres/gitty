# External Integrations

## System Dependencies

### Git CLI

**Service:** System-installed `git` binary
**Purpose:** All write operations — pull, fetch, checkout, rebase, merge, stash, clean, reset (ADR-0001)
**Implementation:** Not yet built. Will shell-out via `std::process::Command` in `gitty-core`
**Configuration:** Uses user's existing git config (`~/.gitconfig`, repo-level `.git/config`)
**Authentication:** Delegates to user's configured credential helpers (SSH keys, credential manager)

### libgit2 (via git2 crate)

**Service:** Embedded Git library (Rust bindings)
**Purpose:** All read operations — status, log, branch info, ref inspection, diff (ADR-0001)
**Implementation:** Not yet added to dependencies. Will be a `gitty-core` dependency
**Configuration:** N/A — reads from `.git` directory directly

## Tauri Plugins

### tauri-plugin-opener

**Purpose:** Open files and URLs with the system default handler
**Location:** Registered in `src-tauri/src/lib.rs` via `.plugin(tauri_plugin_opener::init())`
**Configuration:** Permission granted in `src-tauri/capabilities/default.json` as `opener:default`

## Filesystem

### Config File

**Purpose:** Persistent user configuration (Workspace definitions, Macros, Scheduler rules)
**Location (planned):** Platform-specific via `dirs::config_dir()` — `%APPDATA%\gitty\` (Windows), `~/.config/gitty/` (Linux), `~/Library/Application Support/gitty/` (macOS)
**Implementation:** Not yet built

### Lock File

**Purpose:** Prevent CLI and GUI from running concurrent operations on the same Repository
**Location (planned):** Inside Config directory
**Implementation:** Not yet built

## API Integrations

_None — Gitty is fully local with no cloud/API dependencies in v1._

## Webhooks

_None._

## Background Jobs

### Scheduler (planned)

**Purpose:** Run Macros (default: `git fetch --all`) on configurable triggers (time-of-day, day-of-week, power source)
**Location:** Will live in `gitty-core`, consumed by both CLI and Tauri
**Implementation:** Not yet built
