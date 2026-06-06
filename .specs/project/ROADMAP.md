# Roadmap

## Milestone 1: Foundation

> Core infrastructure — Config, Scan Roots, Repository discovery. No UI yet.

- **Config system** — Read/write Config file, platform-aware paths, schema
- **Scan Root management** — Add/remove Scan Roots, recursive `.git` discovery
- **Repository registry** — UUID assignment, path tracking, persistence
- **Lock mechanism** — File-level locks preventing concurrent operations
- **Git read layer** — git2-based status, branch info, log, ref inspection
- **Git write layer** — Shell-out execution for pull, fetch, checkout, etc.

## Milestone 2: CLI ✓

> Standalone CLI binary exposing core operations via `clap`.

- ✓ **CLI scaffold** — `gitty-cli` crate with clap, subcommands
- ✓ **Workspace commands** — `gitty scan`, `gitty list`, `gitty status`
- ✓ **Operation commands** — `gitty fetch`, `gitty pull`, `gitty checkout`
- ✓ **Group/Tag commands** — `gitty group` (list/create/rename/delete/assign/tree), `gitty tag` (list/add/remove), `gitty filter` (--group/--tag)
- ✓ **Macro commands** — `gitty macro` (list/define/delete/show/run with --group/--tag/--repo selectors)

## Milestone 3: Desktop Shell ✓

> Tauri app with navigation, layout, and Workspace dashboard.

- ✓ **Tauri command bridge** — 10 IPC commands exposing gitty-core to frontend
- ✓ **App shell** — Sidebar navigation, main content area, status bar, bottom bar
- ✓ **Design system** — CSS custom properties from DESIGN.md, warm cream canvas, hairline depth
- ✓ **Workspace dashboard** — Repository table with status badges, stats cards, scan dialog, fetch all
- ✓ **Repository detail** — Branch info, status, tracking, commits, tags, fetch/pull actions
- ✓ **Scan Root management UI** — Settings page with add/remove/rescan

## Milestone 4: Organization & Operations (core + CLI ✓, GUI → M6)

> Groups, Tags, Macros (with scripting), and bulk operations in both CLI and GUI.

### Core + CLI (done)

- ✓ **Group management** — Create, rename, delete, move, nest (arbitrary depth), assign Repositories, tree view, cycle detection
- ✓ **Tag management** — Create, assign, remove, filter by Tags, Favorite built-in
- ✓ **Macro builder** — Define Steps (Git Operations + Shell Commands), variables, conditions, rollback
- ✓ **Macro scripting** — Variables (HashMap), conditions (step skipping), rollback (undo on failure), confirmations (flag)
- ✓ **Macro execution** — Run Macros against selections (All/Single/Group/Tag/Multiple), Job tracking
- ✓ **Job monitoring** — Per-step status, error reporting, rollback execution

### GUI (moved to M6 — D85)

_Outstanding GUI work absorbed into Milestone 6 sub-feature `m6-gui-completion`._

## Milestone 5: Health, Dashboard & Automation (core + CLI ✓, GUI polish → M6)

> Health Checks, Workspace Health score, Change Dashboard, Scheduler.

- ✓ **Health Check engine** — Pluggable checks (freshness, divergence, dirty, detached)
- ✓ **Workspace Health dashboard** — Aggregate score, drill-down per Repository (core + CLI)
- ✓ **Change Dashboard** — What Changed view: commits, authors, repos over time windows (24h, 7d, 30d), grouping by Author/Repository/Branch (core + CLI)
- ✓ **Scheduler** — Background Macro execution on configurable triggers (core + CLI + GUI tokio task)
- ✓ **Notifications** — Surface critical health warnings (core + CLI)

_GUI polish for health, changes, scheduler settings, and notification panel included in M6 (D86)._

## Milestone 6: Polish & Ship

> Complete GUI, refine UX, harden errors, package for distribution, document.

Split into 3 sub-features (D72):

### Sub-feature: `m6-gui-completion`

> Complete all outstanding GUI work from M4 + polish M5 GUI pages.

**From M4 (D85):**
- **Sidebar Group tree** — VS Code-style explorer: collapsible Group tree with Repositories (D27)
- **Groups admin page** — CRUD for Groups (create/rename/delete/move) (D28)
- **Repository detail enhancements** — Group assignment dropdown (D29), inline Tag editor (D30)
- **Macro builder page** — Visual step editor with Git Op picker, Shell input, conditions, rollback, variables (D32)
- **Macro execution** — Selection dialog (All/Group/Tag/individual) + inline Job results panel (D33+D34)

**From M5 (D86):**
- **Health dashboard page** — GUI for Workspace Health score + per-repo drill-down
- **Changes page** — GUI for What Changed view with time windows and grouping
- **Scheduler settings UI** — GUI for scheduler config in Settings
- **Notification panel** — In-app notification display

### Sub-feature: `m6-polish`

> Design system compliance, error handling, UX refinement.

- **Design system token-complete + audit** — Every DESIGN.md token as CSS custom property; audit all components (D73)
- **Self-host fonts** — Bundle Inter + JetBrains Mono, remove CDN, CSP-clean (D74)
- **Toast + contextual error system** — Auto-dismissing toasts for transient errors, inline with recovery hints for persistent (D77)
- **AppError recovery hints** — `hint` field in DTO populated from CoreError metadata (D78)
- **Macro-level retry** — Optional retry config on Step; Git Ops + Network errors only (D76)

### Sub-feature: `m6-ship`

> Packaging, signing, CI, documentation.

- **GitHub Actions release workflow** — Matrix build: Windows NSIS, macOS DMG, Linux AppImage; triggered on version tags (D79)
- **Auto-generated release notes** — git-cliff from conventional commits (D80)
- **Windows code signing** — SignPath.io integration in CI (D81)
- **macOS ad-hoc signing** — Free codesign + Gatekeeper bypass docs + Homebrew cask tap (D82, ADR-0009)
- **Full docs site** — MkDocs Material, source in `docs/`, deployed to GitHub Pages (D83)
- **CLI reference** — Auto-generated from clap help output (D84)
