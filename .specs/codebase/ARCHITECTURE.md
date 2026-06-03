# Architecture

**Pattern:** Tauri 2 desktop app + standalone CLI — Rust backend (gitty-core) + web frontend (SvelteKit SPA). Both binaries share domain logic and persist to the same Config file with file-level locking.

## High-Level Structure

```
┌────────────────────────────────────────────────┐
│              SvelteKit SPA (Frontend)           │
│  Svelte 5 runes + TypeScript, port 1420        │
│  invoke() ──────────────────────────┐          │
└─────────────────────────────────────┼──────────┘
                                      │ IPC (Tauri commands)
┌─────────────────────────────────────┼──────────┐
│           Tauri 2 Runtime (src-tauri)│          │
│  ┌──────────────────────────────────▼────────┐ │
│  │  src-tauri/src/lib.rs                     │ │
│  │  31 commands across 8 modules:            │ │
│  │  workspace, groups, tags, macros,         │ │
│  │  health, changes, scheduler, notifications│ │
│  │  AppState (Mutex<Config>) + file-watcher  │ │
│  │  AppError DTOs with typed error codes     │ │
│  └──────────────────┬────────────────────────┘ │
└─────────────────────┼──────────────────────────┘
                      │ depends on
┌─────────────────────▼──────────────────────────┐
│              gitty-core (domain logic)          │
│  config, repository, scan, reconcile, git::read,│
│  git::write, lock, group, tag, macro_def, job,  │
│  selection, execution, health, health_cache,     │
│  changes, scheduler (runner + daemon),           │
│  notification, power                             │
└───────────────────────┬────────────────────────┘
                        │ depends on
┌───────────────────────▼────────────────────────┐
│              gitty-cli (standalone CLI)          │
│  clap subcommands: scan, list, status, fetch,   │
│  pull, checkout, group, tag, filter, macro,      │
│  health, scheduler, notification                 │
└────────────────────────────────────────────────┘
```

## Architecture (ADR-0002)

Cargo workspace with 3 crates:

```
gitty/
├── crates/
│   ├── gitty-core/    # Pure domain logic — no framework deps
│   └── gitty-cli/     # clap-based CLI binary
└── src-tauri/         # Tauri desktop app (thin shell over core)
```

Both `gitty-cli` and `src-tauri` depend on `gitty-core`. They are **independent processes** sharing the same Config file with file-level Locks. No daemon, no IPC between CLI and GUI.

## Patterns

### Tauri Command + DTO Pattern

**Location:** `src-tauri/src/commands/`
**Purpose:** Frontend-to-backend IPC with type-safe DTOs
**Implementation:** `#[tauri::command]` functions return `Result<DtoType, AppError>`. 31 commands registered via `invoke_handler(generate_handler![...])`. Config access serialized through `AppState` (Mutex-backed managed state with file-watcher for external changes).

### Managed State Pattern (ADR-0007)

**Location:** `src-tauri/src/state.rs`
**Purpose:** Thread-safe config access with auto-reload on external changes
**Implementation:** `AppState` wraps `Mutex<Config>` + config file path. `with_config_write()` acquires the lock, runs the closure, and auto-saves. A `notify` file-watcher detects external config changes and emits `config-changed` events to the frontend.

### Structured Error DTO Pattern

**Location:** `src-tauri/src/error.rs`
**Purpose:** Typed error responses for frontend consumption
**Implementation:** `AppError` with `code` (string) and `message` fields. Maps every `CoreError` variant to a named code (e.g., `group_not_found`, `lock_contention`).

### Health Check Trait Pattern

**Location:** `crates/gitty-core/src/health.rs`
**Purpose:** Extensible health evaluation
**Implementation:** `trait HealthCheck` with `id()` and `evaluate(status, thresholds, now)`. Four built-in checks: `StaleCheck`, `DivergedCheck`, `DirtyCheck`, `DetachedCheck`. `evaluate_workspace()` aggregates per-repo results into a score.

### Scheduler Daemon Pattern

**Location:** `crates/gitty-core/src/scheduler/daemon.rs`
**Purpose:** Cross-platform background scheduler
**Implementation:** Unix: `daemonize` crate (fork+detach). Windows: `DETACHED_PROCESS` flag on child process. PID file for single-instance enforcement with stale detection.

### Domain Model Pattern

**Location:** `crates/gitty-core/src/`
**Purpose:** Pure domain logic shared by CLI and GUI
**Key types:** `Config`, `Workspace`, `Repository`, `Group`, `MacroDef`, `Step`, `Job`, `Selection`, `HealthThresholds`, `SchedulerConfig`, `NotificationConfig`, `ChangeEntry`
**Key traits:** `HealthCheck` (extensible checks). All persisted types derive `Serialize + Deserialize`.

### SPA Mode

**Location:** `src/routes/+layout.ts`
**Purpose:** Disable SSR for Tauri compatibility
**Implementation:** `export const ssr = false` with `adapter-static` fallback to `index.html`

### Capability-Based Security

**Location:** `src-tauri/capabilities/default.json`
**Purpose:** Least-privilege permission model
**Implementation:** Only `core:default` and `opener:default` granted to the main window

## Data Flow

### Read (GUI)

```
User action → Svelte $state update → invoke("list_repositories")
  → Tauri IPC → state.config() (MutexGuard) → gitty-core
  → git2 / config → RepoDto → Svelte $state update → DOM
```

### Write (GUI)

```
User action → invoke("fetch_repo", { repoId })
  → Tauri IPC → state.config() → gitty-core
  → GitBinary::fetch() → shell-out to git CLI → OpResultDto → DOM
```

### Config Mutation (GUI)

```
User action → invoke("create_group", { name })
  → Tauri IPC → state.with_config_write(|config| { ... })
  → gitty-core mutation → auto-save to disk → Result → DOM
```

### CLI

```
gitty fetch → clap parse → Config::load() → GitBinary::run_batch_locked()
  → per-repo lock → git CLI shell-out → print results → ExitCode
```

### Scheduler Loop

```
tick() → Config::load() → should_run() check (power + time + interval)
  → execute_macro (default fetch) → evaluate_workspace → health_cache::save
  → generate_health_notification → config.save() → sleep 30s → tick()
```

## Code Organization

**Approach:** Layer-based (frontend/backend split enforced by Tauri architecture)

**Module boundaries:**
- Frontend (`src/`): SvelteKit routes and components — SPA, design system via CSS custom properties
  - `src/lib/types/`: TypeScript DTO interfaces + invoke wrappers (health.ts, changes.ts, notifications.ts, workspace.ts)
  - `src/lib/components/`: AppShell, Sidebar (group tree), StatusBar, BottomBar, NotificationPanel
  - `src/routes/`: Pages — dashboard, health, changes, settings, repo detail, groups, macros
- Backend (`src-tauri/`): Thin Tauri shell — DTOs, state, error mapping, command handlers
  - `src-tauri/src/state.rs`: AppState (Mutex<Config> + file-watcher)
  - `src-tauri/src/error.rs`: AppError with CoreError mapping
  - `src-tauri/src/commands/`: 8 modules (workspace, groups, tags, macros, health, changes, scheduler, notifications)
- Core (`crates/gitty-core/`): All domain logic — no framework dependencies
  - Config, repository, scan, reconcile, git (read + write), lock
  - Group, tag, macro_def, job, selection, execution
  - Health, health_cache, changes, scheduler (runner + daemon), notification, power
- CLI (`crates/gitty-cli/`): clap subcommands, formatting, user-facing output
- The frontend and Tauri backend communicate exclusively through IPC commands
- The CLI and Tauri app share 100% of domain logic via `gitty-core`
