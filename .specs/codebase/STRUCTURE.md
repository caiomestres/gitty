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
│   ├── features/             # Feature specs (discovery, git-write)
│   └── codebase/             # Brownfield analysis (this directory)
├── .vscode/                  # Editor configuration
├── crates/
│   ├── gitty-core/           # Pure domain logic (no framework deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Module declarations + re-exports
│   │       ├── config/mod.rs     # Config load/save, schema versioning
│   │       ├── config/paths.rs   # Platform config directory resolution
│   │       ├── error.rs          # CoreError enum (thiserror)
│   │       ├── repository.rs     # Repository, Workspace, ScanRoot
│   │       ├── scan.rs           # Recursive .git discovery (walkdir)
│   │       ├── reconcile.rs      # Scan result → registry reconciliation
│   │       ├── git/mod.rs        # Git module declarations
│   │       ├── git/read.rs       # git2-based status/branch/log
│   │       ├── git/write.rs      # Shell-out runner, batch ops, locking
│   │       ├── lock.rs           # Per-repo PID lock files (ADR-0006)
│   │       ├── group.rs          # Group CRUD, tree ops, Ungrouped
│   │       ├── tag.rs            # Tag add/remove/filter, Favorite
│   │       ├── macro_def.rs      # MacroDef, Step, GitOp, ShellStep
│   │       ├── job.rs            # Job, JobStatus, StepResult
│   │       ├── selection.rs      # Selection enum + resolve()
│   │       └── execution.rs      # Macro execution engine
│   └── gitty-cli/            # Standalone CLI binary (clap)
│       ├── Cargo.toml
│       └── src/main.rs       # 10 subcommands: scan/list/status/fetch/pull/checkout/group/tag/filter/macro
├── docs/
│   ├── adr/                  # 6 Architecture Decision Records
│   └── agents/               # Agent configuration docs
├── src/                      # SvelteKit frontend
│   ├── app.html              # HTML shell
│   ├── lib/
│   │   ├── styles/
│   │   │   ├── tokens.css        # Design system CSS custom properties
│   │   │   └── global.css        # Reset + base typography
│   │   ├── types/
│   │   │   └── workspace.ts      # DTO TypeScript interfaces
│   │   ├── components/
│   │   │   ├── AppShell.svelte    # Layout wrapper (sidebar + main + bars)
│   │   │   ├── Sidebar.svelte     # Navigation sidebar
│   │   │   ├── StatusBar.svelte   # Top status bar
│   │   │   └── BottomBar.svelte   # Bottom workspace health bar
│   │   └── smoke.test.ts     # Vitest smoke test
│   └── routes/
│       ├── +layout.ts        # SSR disabled (SPA mode)
│       ├── +layout.svelte    # AppShell wrapper + global styles
│       ├── +page.svelte      # Workspace dashboard
│       ├── repo/[id]/
│       │   └── +page.svelte  # Repository detail view
│       └── settings/
│           └── +page.svelte  # Scan Root management
├── src-tauri/                # Tauri desktop app (workspace member)
│   ├── capabilities/
│   │   └── default.json      # Permission definitions
│   ├── icons/                # App icons (all platforms)
│   ├── src/
│   │   ├── lib.rs            # 10 Tauri commands + DTOs + config mutex
│   │   └── main.rs           # Binary entry point
│   ├── build.rs              # Tauri build script
│   ├── Cargo.toml            # Depends on gitty-core, uuid
│   └── tauri.conf.json       # Tauri configuration (CSP configured)
├── static/                   # Static assets (SVGs, favicon)
├── Cargo.toml                # Workspace manifest
├── AGENTS.md                 # Agent instructions
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
**Key files:** `src/lib.rs` (module declarations + re-exports), 14 source modules
**Depends on:** `serde`, `serde_json`, `uuid`, `git2`, `walkdir`, `dirs`, `dunce`, `thiserror`, `time`
**Test count:** 57 unit tests + 13 integration tests

### CLI (`crates/gitty-cli/`)

**Purpose:** Standalone CLI binary for headless/terminal use
**Key files:** `src/main.rs` (clap-based subcommand router, 10 commands)
**Depends on:** `gitty-core`, `clap`, `anyhow`, `uuid`

### Tauri App (`src-tauri/`)

**Purpose:** Desktop application — thin shell over gitty-core
**Key files:** `src/lib.rs` (10 IPC commands, DTO structs, config mutex), `src/main.rs` (entry point)
**Depends on:** `gitty-core`, `tauri`, `tauri-plugin-opener`, `uuid`

### Frontend (`src/`)

**Purpose:** SvelteKit SPA serving as the Tauri webview content
**Key files:** 4 routes (dashboard, repo detail, settings, layout), 4 components (AppShell, Sidebar, StatusBar, BottomBar), design system (tokens.css, global.css), DTO types
**Patterns:** Svelte 5 runes ($state, $derived, $effect), invoke() IPC

### Documentation (`docs/`)

**Purpose:** Architecture decisions and agent configuration
**Key files:** 6 ADRs, issue tracker config, triage labels

## Where Things Live

**IPC Commands:**
- Definition: `src-tauri/src/lib.rs` (`#[tauri::command]`)
- DTOs: `src-tauri/src/lib.rs` (RepoDto, RepoStatusDto, etc.)
- Frontend types: `src/lib/types/workspace.ts`
- Invocation: route pages via `invoke()`
- Permissions: `src-tauri/capabilities/default.json`

**Domain Logic:**
- Groups: `crates/gitty-core/src/group.rs` (methods on Workspace)
- Tags: `crates/gitty-core/src/tag.rs` (methods on Workspace)
- Macros: `crates/gitty-core/src/macro_def.rs` (definition + CRUD)
- Execution: `crates/gitty-core/src/execution.rs` (macro runner)
- Selection: `crates/gitty-core/src/selection.rs` (target resolver)

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
