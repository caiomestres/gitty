# Project Structure

**Root:** `gitty/`

## Directory Tree

```
gitty/
├── .agents/                  # Agent skills (LLM-agnostic)
│   └── skills/               # Skill definitions (SKILL.md + references)
├── .husky/                   # Git hooks (pre-commit → lint-staged)
├── .specs/                   # Spec-driven development artifacts
│   ├── project/              # PROJECT.md, ROADMAP.md, STATE.md
│   ├── features/             # Feature specs (m5-health-automation, etc.)
│   └── codebase/             # Brownfield analysis (this directory)
├── .vscode/                  # Editor configuration
├── crates/
│   ├── gitty-core/           # Pure domain logic (no framework deps)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            # Module declarations + re-exports + scan_and_reconcile
│   │   │   ├── config/mod.rs     # Config load/save, schema versioning
│   │   │   ├── config/paths.rs   # Platform config directory resolution
│   │   │   ├── error.rs          # CoreError enum (thiserror)
│   │   │   ├── repository.rs     # Repository, Workspace, ScanRoot
│   │   │   ├── scan.rs           # Recursive .git discovery (walkdir)
│   │   │   ├── reconcile.rs      # Scan result → registry reconciliation
│   │   │   ├── git/mod.rs        # Git module declarations
│   │   │   ├── git/read.rs       # git2-based status/branch/log
│   │   │   ├── git/write.rs      # Shell-out runner, batch ops, locking
│   │   │   ├── lock.rs           # Per-repo PID lock files (ADR-0006)
│   │   │   ├── group.rs          # Group CRUD, tree ops, Ungrouped
│   │   │   ├── tag.rs            # Tag add/remove/filter, Favorite
│   │   │   ├── macro_def.rs      # MacroDef, Step, GitOp, ShellStep
│   │   │   ├── job.rs            # Job, JobStatus, StepResult
│   │   │   ├── selection.rs      # Selection enum + resolve()
│   │   │   ├── execution.rs      # Macro execution engine
│   │   │   ├── health.rs         # HealthCheck trait + 4 checks + evaluation
│   │   │   ├── health_cache.rs   # Atomic health.json cache with file locking
│   │   │   ├── changes.rs        # Change Dashboard (git2 revwalk, grouping)
│   │   │   ├── scheduler/        # Scheduler module
│   │   │   │   ├── mod.rs        # Data models, trigger logic, should_run/record_run
│   │   │   │   ├── runner.rs     # tick(), health_poll(), run_loop()
│   │   │   │   └── daemon.rs     # PID file, daemonize, cross-platform process mgmt
│   │   │   ├── notification.rs   # Notification generation + purge
│   │   │   └── power.rs          # Battery state detection (battery crate)
│   │   └── tests/
│   │       └── integration_m5.rs # Cross-module integration tests
│   └── gitty-cli/            # Standalone CLI binary (clap)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # CLI struct definitions + dispatch (~220 lines)
│           └── commands/
│               ├── mod.rs            # Shared helpers (resolve_group_id)
│               ├── workspace.rs      # scan/list/status/fetch/pull/checkout
│               ├── group.rs          # group list/create/rename/delete/assign/tree
│               ├── tag.rs            # tag list/add/remove
│               ├── filter.rs         # filter --group/--tag
│               ├── macros.rs         # macro list/define/delete/show/run
│               ├── health.rs         # health [--repo]
│               ├── scheduler.rs      # scheduler start/stop/status
│               └── notification.rs   # notification show/set
├── docs/
│   ├── adr/                  # 8 Architecture Decision Records
│   └── agents/               # Agent configuration docs
├── src/                      # SvelteKit frontend
│   ├── app.html              # HTML shell
│   ├── lib/
│   │   ├── styles/
│   │   │   ├── tokens.css        # Design system CSS custom properties
│   │   │   └── global.css        # Reset + base typography
│   │   ├── types/
│   │   │   ├── workspace.ts      # Workspace/repo DTO interfaces
│   │   │   ├── health.ts         # Health DTO interfaces + invoke wrappers
│   │   │   ├── changes.ts        # Changes DTO interfaces + invoke wrappers
│   │   │   └── notifications.ts  # Notification DTO interfaces + invoke wrappers
│   │   ├── components/
│   │   │   ├── AppShell.svelte       # Layout wrapper (top bar + sidebar + main + bottom bar)
│   │   │   ├── Sidebar.svelte        # Navigation + Group tree explorer
│   │   │   ├── StatusBar.svelte      # Top status bar
│   │   │   ├── BottomBar.svelte      # Bottom workspace health bar
│   │   │   └── NotificationPanel.svelte  # Bell icon + notification dropdown
│   │   └── smoke.test.ts     # Vitest smoke test
│   └── routes/
│       ├── +layout.ts        # SSR disabled (SPA mode)
│       ├── +layout.svelte    # AppShell wrapper + global styles
│       ├── +page.svelte      # Workspace dashboard
│       ├── repo/[id]/
│       │   └── +page.svelte  # Repository detail view
│       ├── health/
│       │   └── +page.svelte  # Workspace Health dashboard
│       ├── changes/
│       │   └── +page.svelte  # Change Dashboard (commits, grouping, time windows)
│       ├── settings/
│       │   └── +page.svelte  # Scan Roots + Scheduler + Notification settings
│       ├── groups/
│       │   └── +page.svelte  # Groups admin page
│       └── macros/
│           └── +page.svelte  # Macros page
├── src-tauri/                # Tauri desktop app (workspace member)
│   ├── capabilities/
│   │   └── default.json      # Permission definitions
│   ├── icons/                # App icons (all platforms)
│   ├── src/
│   │   ├── lib.rs            # Tauri setup, plugin registration, scheduler/health threads
│   │   ├── main.rs           # Binary entry point
│   │   ├── state.rs          # AppState (Mutex<Config> + file-watcher)
│   │   ├── error.rs          # AppError DTOs with CoreError mapping
│   │   └── commands/
│   │       ├── mod.rs            # Shared DTOs (RepoDto) + helpers (parse_uuid, find_repo)
│   │       ├── workspace.rs      # list/status/scan/remove/fetch/pull/checkout/fetch_all/pull_all
│   │       ├── groups.rs         # list/create/rename/delete/move/assign/tree
│   │       ├── tags.rs           # list/add/remove
│   │       ├── macros.rs         # list/get/define/delete/run
│   │       ├── health.rs         # get_workspace_health/get_repository_health/refresh_health
│   │       ├── changes.rs        # get_changes
│   │       ├── scheduler.rs      # get_scheduler_status/set_scheduler_config
│   │       └── notifications.rs  # get/mark_read/get_config/set_config
│   ├── build.rs              # Tauri build script
│   ├── Cargo.toml            # Depends on gitty-core, tauri, notify, uuid, time
│   └── tauri.conf.json       # Tauri configuration (CSP configured)
├── static/                   # Static assets (SVGs, favicon)
├── Cargo.toml                # Workspace manifest (3 members)
├── AGENTS.md                 # Agent instructions + skill catalog
├── CONTEXT.md                # Domain glossary
├── DESIGN.md                 # UI design system
├── eslint.config.js          # ESLint flat config
├── package.json              # Frontend deps + scripts
├── .prettierrc               # Prettier configuration
├── svelte.config.js          # SvelteKit config (adapter-static)
├── tsconfig.json             # TypeScript config (strict)
└── vite.config.js            # Vite config (Tauri-tailored)
```

## Module Organization

### Core Domain (`crates/gitty-core/`)

**Purpose:** Pure domain logic — no Tauri, no CLI framework dependencies
**Key files:** `src/lib.rs` (module declarations + re-exports), 20 source modules
**Depends on:** `serde`, `serde_json`, `uuid`, `git2`, `walkdir`, `dirs`, `dunce`, `thiserror`, `time`, `fs2`, `battery`, `daemonize` (Unix only)
**Test count:** 82 unit tests + 12 integration tests

### CLI (`crates/gitty-cli/`)

**Purpose:** Standalone CLI binary for headless/terminal use
**Key files:** `src/main.rs` (CLI definitions + dispatch), `src/commands/` (8 handler modules, 13 commands)
**Depends on:** `gitty-core`, `clap`, `anyhow`, `uuid`, `time`

### Tauri App (`src-tauri/`)

**Purpose:** Desktop application — thin shell over gitty-core
**Key files:** `src/lib.rs` (setup + command registration), `src/state.rs` (AppState), `src/error.rs` (AppError), `src/commands/` (8 command modules, 31 IPC commands)
**Depends on:** `gitty-core`, `tauri`, `tauri-plugin-opener`, `uuid`, `time`, `notify`
**Test count:** 18 unit tests

### Frontend (`src/`)

**Purpose:** SvelteKit SPA serving as the Tauri webview content
**Key files:** 7 routes (dashboard, health, changes, settings, repo detail, groups, macros), 5 components (AppShell, Sidebar, StatusBar, BottomBar, NotificationPanel), 4 type modules, design system (tokens.css, global.css)
**Patterns:** Svelte 5 runes ($state, $derived, $effect), invoke() IPC, onMount for initial loads

### Documentation (`docs/`)

**Purpose:** Architecture decisions and agent configuration
**Key files:** 8 ADRs, issue tracker config, triage labels

## Where Things Live

**IPC Commands:**
- Definition: `src-tauri/src/commands/*.rs` (31 `#[tauri::command]` functions)
- Registration: `src-tauri/src/lib.rs` (`generate_handler![...]`)
- DTOs: `src-tauri/src/commands/*.rs` (per-module DTOs)
- Shared DTOs: `src-tauri/src/commands/mod.rs` (RepoDto, parse_uuid, find_repo)
- Frontend types: `src/lib/types/*.ts` (health.ts, changes.ts, notifications.ts, workspace.ts)
- Invocation: route pages via `invoke()`
- Permissions: `src-tauri/capabilities/default.json`

**Domain Logic:**
- Config: `crates/gitty-core/src/config/` (load/save, paths, schema versioning)
- Repository: `crates/gitty-core/src/repository.rs` (Workspace, Repository, ScanRoot)
- Groups: `crates/gitty-core/src/group.rs` (methods on Workspace)
- Tags: `crates/gitty-core/src/tag.rs` (methods on Workspace)
- Macros: `crates/gitty-core/src/macro_def.rs` (definition + CRUD)
- Execution: `crates/gitty-core/src/execution.rs` (macro runner)
- Selection: `crates/gitty-core/src/selection.rs` (target resolver)
- Health: `crates/gitty-core/src/health.rs` (trait + checks + evaluation)
- Health Cache: `crates/gitty-core/src/health_cache.rs` (atomic file persistence)
- Changes: `crates/gitty-core/src/changes.rs` (git2 revwalk + grouping)
- Scheduler: `crates/gitty-core/src/scheduler/` (models, trigger logic, runner, daemon)
- Notifications: `crates/gitty-core/src/notification.rs` (generation + purge)
- Power: `crates/gitty-core/src/power.rs` (battery state)

**Configuration:**
- Workspace: `Cargo.toml` (root)
- Tauri: `src-tauri/tauri.conf.json`
- Vite: `vite.config.js`
- SvelteKit: `svelte.config.js`
- TypeScript: `tsconfig.json`
- ESLint: `eslint.config.js`
- Prettier: `.prettierrc`

**Design System:**
- Tokens: `src/lib/styles/tokens.css`
- Reset/base: `src/lib/styles/global.css`
- Spec: `DESIGN.md`

**Domain Knowledge:**
- Glossary: `CONTEXT.md`
- Decisions: `docs/adr/`
- Design system: `DESIGN.md`
