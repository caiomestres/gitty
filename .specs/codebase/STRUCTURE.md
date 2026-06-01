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
│   └── codebase/             # Brownfield analysis (this directory)
├── .vscode/                  # Editor configuration
├── crates/
│   ├── gitty-core/           # Pure domain logic (no framework deps)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── gitty-cli/            # Standalone CLI binary (clap)
│       ├── Cargo.toml
│       └── src/main.rs
├── docs/
│   ├── adr/                  # Architecture Decision Records
│   └── agents/               # Agent configuration docs
├── src/                      # SvelteKit frontend
│   ├── app.html              # HTML shell
│   ├── lib/                  # Shared utilities + tests
│   │   └── smoke.test.ts     # Vitest smoke test
│   └── routes/
│       ├── +layout.ts        # SSR disabled (SPA mode)
│       └── +page.svelte      # Root page (scaffold)
├── src-tauri/                # Tauri desktop app (workspace member)
│   ├── capabilities/
│   │   └── default.json      # Permission definitions
│   ├── icons/                # App icons (all platforms)
│   ├── src/
│   │   ├── lib.rs            # Tauri builder + commands
│   │   └── main.rs           # Binary entry point
│   ├── build.rs              # Tauri build script
│   ├── Cargo.toml            # Depends on gitty-core
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
**Key files:** `src/lib.rs` (currently scaffold, will hold all domain modules)
**Depends on:** `serde`, `serde_json`, `uuid`

### CLI (`crates/gitty-cli/`)

**Purpose:** Standalone CLI binary for headless/terminal use
**Key files:** `src/main.rs` (clap-based subcommand router)
**Depends on:** `gitty-core`, `clap`

### Tauri App (`src-tauri/`)

**Purpose:** Desktop application — thin shell over gitty-core
**Key files:** `src/lib.rs` (builder + IPC commands), `src/main.rs` (entry point)
**Depends on:** `gitty-core`, `tauri`, `tauri-plugin-opener`

### Frontend (`src/`)

**Purpose:** SvelteKit SPA serving as the Tauri webview content
**Key files:** `routes/+page.svelte`, `routes/+layout.ts`, `lib/` (shared code + tests)

### Documentation (`docs/`)

**Purpose:** Architecture decisions and agent configuration
**Key files:** 3 ADRs, issue tracker config, triage labels

## Where Things Live

**IPC Commands:**
- Definition: `src-tauri/src/lib.rs` (`#[tauri::command]`)
- Invocation: `src/routes/+page.svelte` (`invoke()`)
- Permissions: `src-tauri/capabilities/default.json`

**Configuration:**
- Workspace: `Cargo.toml` (root)
- Tauri: `src-tauri/tauri.conf.json`
- Vite: `vite.config.js`
- SvelteKit: `svelte.config.js`
- TypeScript: `tsconfig.json`
- ESLint: `eslint.config.js`
- Prettier: `.prettierrc`

**Domain Knowledge:**
- Glossary: `CONTEXT.md`
- Decisions: `docs/adr/`
- Design system: `DESIGN.md`
