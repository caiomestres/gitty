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
│  │  10 commands: list, status, scan, remove, │ │
│  │  fetch, pull, checkout, fetch_all, pull_all│ │
│  │  DTO structs + config mutex                │ │
│  └──────────────────┬────────────────────────┘ │
└─────────────────────┼──────────────────────────┘
                      │ depends on
┌─────────────────────▼──────────────────────────┐
│              gitty-core (domain logic)          │
│  config, repository, scan, reconcile, git::read,│
│  git::write, lock, group, tag, macro_def, job,  │
│  selection, execution                           │
└───────────────────────┬────────────────────────┘
                        │ depends on
┌───────────────────────▼────────────────────────┐
│              gitty-cli (standalone CLI)          │
│  clap subcommands: scan, list, status, fetch,   │
│  pull, checkout, group, tag, filter, macro       │
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

**Location:** `src-tauri/src/lib.rs`
**Purpose:** Frontend-to-backend IPC with type-safe DTOs
**Implementation:** `#[tauri::command]` functions return `Result<DtoType, String>`, registered via `invoke_handler(generate_handler![...])`. Config access serialized through a `Mutex`.

### Domain Model Pattern

**Location:** `crates/gitty-core/src/`
**Purpose:** Pure domain logic shared by CLI and GUI
**Key types:** `Config`, `Workspace`, `Repository`, `Group`, `MacroDef`, `Step`, `Job`, `Selection`
**Key traits:** All persisted types derive `Serialize + Deserialize`

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
  → Tauri IPC → with_config_read(|config| ...) → gitty-core
  → git2 / config → RepoDto → Svelte $state update → DOM
```

### Write (GUI)

```
User action → invoke("fetch_repo", { repoId })
  → Tauri IPC → with_config_read(|config| ...) → gitty-core
  → GitBinary::fetch() → shell-out to git CLI → OpResultDto → DOM
```

### CLI

```
gitty fetch → clap parse → Config::load() → GitBinary::run_batch_locked()
  → per-repo lock → git CLI shell-out → print results → ExitCode
```

## Code Organization

**Approach:** Layer-based (frontend/backend split enforced by Tauri architecture)

**Module boundaries:**
- Frontend (`src/`): SvelteKit routes and components — SPA, design system via CSS custom properties
- Backend (`src-tauri/`): Thin Tauri shell — DTOs, config mutex, command handlers
- Core (`crates/gitty-core/`): All domain logic — no framework dependencies
- CLI (`crates/gitty-cli/`): clap subcommands, formatting, user-facing output
- The frontend and Tauri backend communicate exclusively through IPC commands
- The CLI and Tauri app share 100% of domain logic via `gitty-core`
