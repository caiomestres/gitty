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

## Milestone 4: Organization & Operations (partial — core + CLI done, GUI outstanding)

> Groups, Tags, Macros (with scripting), and bulk operations in both CLI and GUI.

### Core + CLI (done)

- ✓ **Group management** — Create, rename, delete, move, nest (arbitrary depth), assign Repositories, tree view, cycle detection
- ✓ **Tag management** — Create, assign, remove, filter by Tags, Favorite built-in
- ✓ **Macro builder** — Define Steps (Git Operations + Shell Commands), variables, conditions, rollback
- ✓ **Macro scripting** — Variables (HashMap), conditions (step skipping), rollback (undo on failure), confirmations (flag)
- ✓ **Macro execution** — Run Macros against selections (All/Single/Group/Tag/Multiple), Job tracking
- ✓ **Job monitoring** — Per-step status, error reporting, rollback execution

### GUI (outstanding — D23)

- **Tauri refactor** — Managed state + file-watcher (ADR-0007), structured error DTOs, ~13 new IPC commands
- **Sidebar Group tree** — VS Code-style explorer: collapsible Group tree with Repositories (D27)
- **Groups admin page** — CRUD for Groups (create/rename/delete/move) (D28)
- **Repository detail enhancements** — Group assignment dropdown (D29), inline Tag editor (D30)
- **Dashboard Tag filter** — Dropdown above repo table (D31)
- **Macro builder page** — Visual step editor with Git Op picker, Shell input, conditions, rollback, variables (D32)
- **Macro execution** — Selection dialog (All/Group/Tag/individual) + inline Job results panel (D33+D34)

## Milestone 5: Health, Dashboard & Automation

> Health Checks, Workspace Health score, Change Dashboard, Scheduler.

- **Health Check engine** — Pluggable checks (freshness, divergence, dirty, detached)
- **Workspace Health dashboard** — Aggregate score, drill-down per Repository
- **Change Dashboard** — What Changed view: commits, authors, repos over time windows (24h, 7d, 30d), grouping by Author/Repository/Branch
- **Scheduler** — Background Macro execution on configurable triggers
- **Notifications** — Surface critical health warnings

## Milestone 6: Polish & Ship

> UI refinement, error handling, packaging, documentation.

- **Design system implementation** — Full DESIGN.md compliance
- **Error handling** — Graceful failures, retry logic for transient errors
- **Packaging** — Windows installer, macOS DMG, Linux AppImage
- **User documentation** — Getting started, CLI reference
