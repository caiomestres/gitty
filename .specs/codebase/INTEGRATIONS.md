# External Integrations

## System Dependencies

### Git CLI

**Service:** System-installed `git` binary
**Purpose:** All write operations — fetch, pull, checkout (ADR-0001). Scheduler default action is `git fetch --all`.
**Implementation:** `GitBinary::resolve()` locates git on PATH, `std::process::Command` executes with `GIT_TERMINAL_PROMPT=0` and SSH `BatchMode=yes` to prevent interactive prompts.
**Location:** `crates/gitty-core/src/git/write.rs`
**Configuration:** Uses user's existing git config (`~/.gitconfig`, repo-level `.git/config`)
**Authentication:** Delegates to user's configured credential helpers (SSH keys, credential manager)

### libgit2 (via git2 crate)

**Service:** Embedded Git library (Rust bindings), vendored (default-features = false)
**Purpose:** All read operations — status, branch info, ahead/behind, HEAD commit summary, changed files list, commit log (revwalk for Change Dashboard)
**Location:** `crates/gitty-core/src/git/read.rs` (status), `crates/gitty-core/src/changes.rs` (revwalk)
**Configuration:** N/A — reads from `.git` directory directly

### battery crate

**Service:** Cross-platform battery state detection
**Purpose:** Power-aware scheduling — pause scheduler when on battery below threshold
**Location:** `crates/gitty-core/src/power.rs`
**Behavior:** Returns `(on_battery: bool, level: u8)`. Desktops without batteries return `(false, 100)`.

### daemonize crate (Unix only)

**Service:** Process daemonization
**Purpose:** CLI `gitty scheduler start` forks into a background daemon
**Location:** `crates/gitty-core/src/scheduler/daemon.rs`
**Windows alternative:** `DETACHED_PROCESS` + `CREATE_NEW_PROCESS_GROUP` flags via `CommandExt`

## Tauri Plugins

### tauri-plugin-opener

**Purpose:** Open files and URLs with the system default handler
**Location:** Registered in `src-tauri/src/lib.rs` via `.plugin(tauri_plugin_opener::init())`
**Configuration:** Permission granted in `src-tauri/capabilities/default.json` as `opener:default`

## Filesystem

### Config File

**Purpose:** Persistent user configuration (Workspace, Scheduler, Notifications, Notification history)
**Location:** Platform-specific via `dirs::config_dir()` — `%APPDATA%\gitty\config.json` (Windows), `~/.config/gitty/config.json` (Linux), `~/Library/Application Support/gitty/config.json` (macOS)
**Implementation:** `Config::load()` / `Config::save()` with atomic temp+rename writes. Schema version 1 with hard-error on mismatch (ADR-0004).
**File watching:** `notify` crate watches config directory for external changes (CLI modifying config while GUI is open). Emits `config-changed` Tauri event.

### Health Cache File

**Purpose:** Cached workspace health evaluation results
**Location:** `health.json` in the config directory
**Implementation:** `health_cache::save()` / `health_cache::load()` with atomic temp+rename writes and advisory file locking via `fs2`.

### Lock Files

**Purpose:** Prevent CLI and GUI from running concurrent operations on the same Repository
**Location:** `<config_dir>/<repo-uuid>.lock` files
**Implementation:** PID + timestamp JSON, stale detection via process alive check (ADR-0006)

### Scheduler PID File

**Purpose:** Single-instance scheduler enforcement
**Location:** `<config_dir>/scheduler.pid`
**Implementation:** JSON with `pid` and `started_at`. Stale PID detection on both Unix (kill -0) and Windows (OpenProcess + GetExitCodeProcess).

## API Integrations

_None — Gitty is fully local with no cloud/API dependencies in v1._

## Webhooks

_None._

## Background Jobs

### Scheduler (implemented)

**Purpose:** Run Macros (default: `git fetch --all`) on configurable triggers
**Triggers:** Simple (interval) or Advanced (interval + time window + day-of-week constraints, midnight-crossing support)
**Power awareness:** Pauses on battery below configurable threshold (default 20%)
**Implementation:**
- GUI: `std::thread::spawn` in Tauri `setup()`, loops every 30s calling `runner::tick()`
- CLI: Daemon via `gitty scheduler start` (daemonize on Unix, DETACHED_PROCESS on Windows)
- Single-instance via PID file with stale detection

### Health Polling (implemented)

**Purpose:** Periodic health re-evaluation independent of scheduler runs
**Implementation:** Separate `std::thread::spawn` in Tauri `setup()`, calls `runner::health_poll()` every 5 minutes
