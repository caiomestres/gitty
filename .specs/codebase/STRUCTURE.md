# Project Structure

**Analyzed:** 2026-06-06

**Root:** `gitty/`

## Directory Tree

```
gitty/
├── .agents/                  # Agent skills (LLM-agnostic)
│   └── skills/               # 33 skill definitions (SKILL.md + references)
├── .github/
│   └── workflows/            # CI, Release, Docs workflows
├── .husky/                   # Git hooks (pre-commit → lint-staged)
├── .specs/                   # Spec-driven development artifacts
│   ├── project/              # PROJECT.md, ROADMAP.md, STATE.md
│   ├── features/             # Feature specs (8 features)
│   └── codebase/             # Brownfield analysis (this directory)
├── .vscode/                  # Editor configuration (settings + extensions)
├── crates/
│   ├── gitty-core/           # Pure domain logic (no framework deps)
│   │   ├── Cargo.toml
│   │   ├── src/              # 26 source modules
│   │   └── tests/            # Integration tests
│   └── gitty-cli/            # Standalone CLI binary (clap)
│       ├── Cargo.toml
│       └── src/              # main.rs + commands/ (8 modules)
├── docs/
│   ├── adr/                  # 9 Architecture Decision Records
│   ├── agents/               # Agent configuration docs
│   ├── stylesheets/          # MkDocs custom styles
│   ├── index.md              # Getting Started
│   ├── concepts.md           # Core concepts documentation
│   ├── cli-reference.md      # CLI command reference
│   └── gui-guide.md          # GUI usage guide
├── homebrew/
│   └── gitty.rb              # Homebrew formula
├── scripts/
│   ├── generate-cli-reference.sh  # CLI reference auto-generation
│   ├── ci-tauri-wrapper.sh        # CI build helper
│   └── ci-macos-tauri-build.sh    # macOS CI build helper
├── site/                     # MkDocs build output (generated)
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
│   │   │   ├── notifications.ts  # Notification DTO interfaces + invoke wrappers
│   │   │   └── scheduler.ts      # Scheduler DTO interfaces + helpers
│   │   ├── stores/
│   │   │   └── toast.svelte.ts   # Toast notification store (Svelte 5 rune-based)
│   │   ├── utils/
│   │   │   ├── error-handling.ts # Error classification + routing (transient/persistent)
│   │   │   └── config-events.ts  # Tauri event listener helpers
│   │   ├── components/
│   │   │   ├── AppShell.svelte       # Layout wrapper (top bar + sidebar + main + bottom bar)
│   │   │   ├── Sidebar.svelte        # Navigation + Group tree explorer (D27)
│   │   │   ├── StatusBar.svelte      # Top status bar
│   │   │   ├── BottomBar.svelte      # Bottom workspace health bar
│   │   │   ├── NotificationPanel.svelte  # Bell icon + notification dropdown
│   │   │   ├── ToastContainer.svelte     # Auto-dismissing toast stack
│   │   │   ├── Dialog.svelte             # Reusable modal dialog
│   │   │   ├── PageError.svelte          # Inline error display with hints
│   │   │   ├── FeedbackBanner.svelte     # Success/error banner component
│   │   │   ├── MacroEditor.svelte        # Visual macro step builder (D32)
│   │   │   ├── MacroRunner.svelte        # Macro execution selection + progress
│   │   │   ├── StepKindEditor.svelte     # Step type picker sub-component
│   │   │   ├── JobResults.svelte         # Macro execution results display
│   │   │   ├── SchedulerSettings.svelte  # Scheduler configuration form
│   │   │   ├── NotificationSettings.svelte  # Notification prefs form
│   │   │   ├── RepoStatusGrid.svelte    # Repository status badge grid
│   │   │   ├── RepoGroupSelect.svelte   # Group assignment dropdown (D29)
│   │   │   ├── RepoTagEditor.svelte     # Inline tag editor (D30)
│   │   │   ├── RepoHealthSection.svelte  # Per-repo health display
│   │   │   └── ChangedFilesList.svelte   # Changed files list component
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
│       │   └── +page.svelte  # Groups admin page (CRUD)
│       └── macros/
│           └── +page.svelte  # Macros page (builder + execution)
├── src-tauri/                # Tauri desktop app (workspace member)
│   ├── capabilities/
│   │   └── default.json      # Permission definitions
│   ├── icons/                # App icons (all platforms)
│   ├── src/
│   │   ├── lib.rs            # Tauri setup, plugin registration, scheduler/health threads
│   │   ├── main.rs           # Binary entry point
│   │   ├── state.rs          # AppState (Mutex<Config> + file-watcher + GitBinary)
│   │   ├── error.rs          # AppError DTOs with CoreError mapping + hints
│   │   └── commands/
│   │       ├── mod.rs            # Shared DTOs (RepoDto) + helpers (parse_uuid, find_repo)
│   │       ├── workspace.rs      # list/status/scan/remove/fetch/pull/checkout/fetch_all/pull_all
│   │       ├── groups.rs         # list/create/rename/delete/move/assign/tree
│   │       ├── tags.rs           # list/add/remove
│   │       ├── macros.rs         # list/get/define/update/delete/run
│   │       ├── health.rs         # get_workspace_health/get_repository_health/refresh_health
│   │       ├── changes.rs        # get_changes
│   │       ├── scheduler.rs      # get_scheduler_config/get_scheduler_status/set_scheduler_config
│   │       └── notifications.rs  # get/mark_read/get_config/set_config
│   ├── build.rs              # Tauri build script
│   ├── Cargo.toml            # Depends on gitty-core, tauri, notify, uuid, time
│   └── tauri.conf.json       # Tauri configuration (CSP configured)
├── static/
│   └── fonts/                # Self-hosted fonts (Inter 400/500/600, JetBrains Mono 400/500)
├── Cargo.toml                # Workspace manifest (3 members)
├── Taskfile.yml              # Task runner config (frontend + backend checks)
├── AGENTS.md                 # Agent instructions + skill catalog
├── CONTEXT.md                # Domain glossary
├── DESIGN.md                 # UI design system
├── mkdocs.yml                # Documentation site config
├── eslint.config.js          # ESLint flat config
├── package.json              # Frontend deps + scripts
├── .prettierrc               # Prettier configuration
├── .prettierignore           # Prettier ignore patterns
├── svelte.config.js          # SvelteKit config (adapter-static)
├── tsconfig.json             # TypeScript config (strict)
└── vite.config.js            # Vite config (Tauri-tailored)
```

## Module Organization

### Core Domain (`crates/gitty-core/`)

**Purpose:** Pure domain logic — no Tauri, no CLI framework dependencies
**Key files:** `src/lib.rs` (module declarations + re-exports), 26 source modules
**Depends on:** `serde`, `serde_json`, `uuid`, `git2`, `walkdir`, `dirs`, `dunce`, `thiserror`, `time`, `battery`, `rayon`, `daemonize` (Unix only)
**Test count:** 155 tests (unit + integration)

### CLI (`crates/gitty-cli/`)

**Purpose:** Standalone CLI binary for headless/terminal use
**Key files:** `src/main.rs` (CLI definitions + dispatch), `src/commands/` (8 handler modules, 13+ commands)
**Depends on:** `gitty-core`, `clap`, `anyhow`, `uuid`, `time`

### Tauri App (`src-tauri/`)

**Purpose:** Desktop application — thin shell over gitty-core
**Key files:** `src/lib.rs` (setup + command registration + scheduler thread), `src/state.rs` (AppState), `src/error.rs` (AppError + hints), `src/commands/` (8 command modules, 38 IPC commands)
**Depends on:** `gitty-core`, `tauri`, `tauri-plugin-opener`, `uuid`, `time`, `notify`
**Test count:** 26 unit tests

### Frontend (`src/`)

**Purpose:** SvelteKit SPA serving as the Tauri webview content
**Key files:** 7 routes, 20 components, 5 type modules, 1 store, 2 utils, design system (tokens.css, global.css)
**Patterns:** Svelte 5 runes ($state, $derived, $effect), invoke() IPC, onMount for initial loads, toast store for transient errors

### Documentation (`docs/`)

**Purpose:** Architecture decisions, agent configuration, and user-facing docs site
**Key files:** 9 ADRs, MkDocs site with 4 pages (Getting Started, Concepts, CLI Reference, GUI Guide)

## Where Things Live

**IPC Commands:**
- Definition: `src-tauri/src/commands/*.rs` (38 `#[tauri::command]` functions)
- Registration: `src-tauri/src/lib.rs` (`generate_handler![...]`)
- DTOs: `src-tauri/src/commands/*.rs` (per-module DTOs)
- Shared DTOs: `src-tauri/src/commands/mod.rs` (RepoDto, parse_uuid, find_repo)
- Frontend types: `src/lib/types/*.ts` (health.ts, changes.ts, notifications.ts, workspace.ts, scheduler.ts)
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
- Process: `crates/gitty-core/src/process.rs` (process management helpers)

**Configuration:**
- Workspace: `Cargo.toml` (root)
- Tauri: `src-tauri/tauri.conf.json`
- Vite: `vite.config.js`
- SvelteKit: `svelte.config.js`
- TypeScript: `tsconfig.json`
- ESLint: `eslint.config.js`
- Prettier: `.prettierrc`
- Task runner: `Taskfile.yml`
- MkDocs: `mkdocs.yml`

**Design System:**
- Tokens: `src/lib/styles/tokens.css`
- Reset/base: `src/lib/styles/global.css`
- Fonts: `static/fonts/` (Inter 400/500/600, JetBrains Mono 400/500 — self-hosted woff2)
- Spec: `DESIGN.md`

**Error Handling:**
- Core errors: `crates/gitty-core/src/error.rs` (CoreError enum)
- IPC errors: `src-tauri/src/error.rs` (AppError with code + hint + transient)
- Frontend routing: `src/lib/utils/error-handling.ts` (transient → toast, persistent → inline)
- Toast display: `src/lib/stores/toast.svelte.ts` + `src/lib/components/ToastContainer.svelte`

**CI/CD:**
- CI: `.github/workflows/ci.yml` (frontend + Rust matrix)
- Release: `.github/workflows/release.yml` (matrix build + git-cliff)
- Docs: `.github/workflows/docs.yml` (MkDocs → GitHub Pages)
- Scripts: `scripts/` (CI helpers, CLI reference generation)

**Domain Knowledge:**
- Glossary: `CONTEXT.md`
- Decisions: `docs/adr/`
- Design system: `DESIGN.md`
