# Architecture

**Pattern:** Tauri desktop app — Rust backend + web frontend (SPA), currently a default scaffold with no feature code.

## High-Level Structure

```
┌─────────────────────────────────────────┐
│              SvelteKit SPA              │
│  (Svelte 5 + TypeScript, port 1420)     │
│         invoke() ──────────────┐        │
└────────────────────────────────┼────────┘
                                 │ IPC (Tauri commands)
┌────────────────────────────────┼────────┐
│           Tauri 2 Runtime      │        │
│  ┌─────────────────────────────▼──────┐ │
│  │         src-tauri/src/lib.rs       │ │
│  │   tauri::Builder + command handlers│ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │       src-tauri/src/main.rs        │ │
│  │   Binary entry point (calls run()) │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

## Planned Architecture (ADR-0002)

The current single-crate layout will evolve into a Cargo workspace:

```
gitty/
├── crates/
│   ├── gitty-core/    # Pure domain logic — no Tauri dependency
│   ├── gitty-cli/     # clap-based CLI binary
│   └── gitty-tauri/   # Tauri desktop app (thin shell over core)
```

Both `gitty-cli` and `gitty-tauri` depend on `gitty-core`. They are independent processes sharing the same Config file with file-level Locks.

## Identified Patterns

### Tauri Command Pattern

**Location:** `src-tauri/src/lib.rs`
**Purpose:** Frontend-to-backend IPC
**Implementation:** `#[tauri::command]` functions registered via `invoke_handler(generate_handler![...])`
**Example:** `greet()` command — takes `&str`, returns `String`

### SPA Mode

**Location:** `src/routes/+layout.ts`
**Purpose:** Disable SSR for Tauri compatibility
**Implementation:** `export const ssr = false` with `adapter-static` fallback to `index.html`

### Capability-Based Security

**Location:** `src-tauri/capabilities/default.json`
**Purpose:** Least-privilege permission model
**Implementation:** Only `core:default` and `opener:default` granted to the main window

## Data Flow

### IPC Call (current scaffold)

```
User action → Svelte event handler → invoke("greet", { name })
  → Tauri IPC bridge → #[tauri::command] fn greet()
  → Return String → Svelte $state update → DOM
```

### Planned Data Flow (ADR-0001)

```
Read operations:  Frontend → Tauri command → gitty-core → git2 → structured data
Write operations: Frontend → Tauri command → gitty-core → shell-out to git CLI → exit code + output
```

## Code Organization

**Approach:** Layer-based (frontend/backend split enforced by Tauri architecture)

**Module boundaries:**
- Frontend (`src/`): SvelteKit routes and components — SPA, no server code
- Backend (`src-tauri/`): Rust binary with Tauri runtime — all system access here
- Static assets (`static/`): SVG logos, favicon
- The two layers communicate exclusively through Tauri IPC commands
