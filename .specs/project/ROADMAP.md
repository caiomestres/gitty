# Roadmap

## Milestone 1: Foundation ✓

> Core infrastructure — Config, Scan Roots, Repository discovery. No UI yet.

- ✓ **Config system** — Read/write Config file, platform-aware paths, schema
- ✓ **Scan Root management** — Add/remove Scan Roots, recursive `.git` discovery
- ✓ **Repository registry** — UUID assignment, path tracking, persistence
- ✓ **Lock mechanism** — File-level locks preventing concurrent operations
- ✓ **Git read layer** — git2-based status, branch info, log, ref inspection
- ✓ **Git write layer** — Shell-out execution for pull, fetch, checkout, etc.

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

## Milestone 4: Organization & Operations (core + CLI ✓, GUI → M6) ✓

> Groups, Tags, Macros (with scripting), and bulk operations in both CLI and GUI.

### Core + CLI (done)

- ✓ **Group management** — Create, rename, delete, move, nest (arbitrary depth), assign Repositories, tree view, cycle detection
- ✓ **Tag management** — Create, assign, remove, filter by Tags, Favorite built-in
- ✓ **Macro builder** — Define Steps (Git Operations + Shell Commands), variables, conditions, rollback
- ✓ **Macro scripting** — Variables (HashMap), conditions (step skipping), rollback (undo on failure), confirmations (flag)
- ✓ **Macro execution** — Run Macros against selections (All/Single/Group/Tag/Multiple), Job tracking
- ✓ **Job monitoring** — Per-step status, error reporting, rollback execution

### GUI (delivered in M6 — D85)

- ✓ Sidebar Group tree, Groups admin, Macro builder, Macro execution, Repo detail enhancements

## Milestone 5: Health, Dashboard & Automation ✓

> Health Checks, Workspace Health score, Change Dashboard, Scheduler.

- ✓ **Health Check engine** — Pluggable checks (freshness, divergence, dirty, detached)
- ✓ **Workspace Health dashboard** — Aggregate score, drill-down per Repository
- ✓ **Change Dashboard** — What Changed view: commits, authors, repos over time windows (24h, 7d, 30d), grouping by Author/Repository/Branch
- ✓ **Scheduler** — Background Macro execution on configurable triggers (simple + advanced, power-aware)
- ✓ **Notifications** — Surface critical health warnings (OS-native + in-app)

## Milestone 6: Polish & Ship ✓

> Complete GUI, refine UX, harden errors, package for distribution, document.

Split into 3 sub-features (D72):

### Sub-feature: `m6-gui-completion` ✓

> Complete all outstanding GUI work from M4 + polish M5 GUI pages.

**From M4 (D85):**
- ✓ **Sidebar Group tree** — VS Code-style explorer: collapsible Group tree with Repositories (D27)
- ✓ **Groups admin page** — CRUD for Groups (create/rename/delete/move) (D28)
- ✓ **Repository detail enhancements** — Group assignment dropdown (D29), inline Tag editor (D30)
- ✓ **Macro builder page** — Visual step editor with Git Op picker, Shell input, conditions, rollback, variables (D32)
- ✓ **Macro execution** — Selection dialog (All/Group/Tag/individual) + inline Job results panel (D33+D34)

**From M5 (D86):**
- ✓ **Health dashboard page** — GUI for Workspace Health score + per-repo drill-down
- ✓ **Changes page** — GUI for What Changed view with time windows and grouping
- ✓ **Scheduler settings UI** — GUI for scheduler config in Settings
- ✓ **Notification panel** — In-app notification display

### Sub-feature: `m6-polish` ✓

> Design system compliance, error handling, UX refinement.

- ✓ **Design system token-complete + audit** — Every DESIGN.md token as CSS custom property; audit all components (D73)
- ✓ **Self-host fonts** — Bundle Inter + JetBrains Mono, remove CDN, CSP-clean (D74)
- ✓ **Toast + contextual error system** — Auto-dismissing toasts for transient errors, inline with recovery hints for persistent (D77)
- ✓ **AppError recovery hints** — `hint` field in DTO populated from CoreError metadata (D78)
- ✓ **Macro-level retry** — Optional retry config on Step; Git Ops + Network errors only (D76)

### Sub-feature: `m6-ship` ✓

> Packaging, signing, CI, documentation.

- ✓ **GitHub Actions CI workflow** — Frontend checks + Rust matrix (Linux/Windows/macOS)
- ✓ **GitHub Actions release workflow** — Matrix build: Windows NSIS, macOS DMG, Linux AppImage; triggered on version tags (D79)
- ✓ **Auto-generated release notes** — git-cliff from conventional commits (D80)
- ✓ **Windows code signing** — SignPath.io integration in CI (D81)
- ✓ **macOS ad-hoc signing** — Free codesign + Gatekeeper bypass docs + Homebrew cask tap (D82, ADR-0009)
- ✓ **Full docs site** — MkDocs Material, source in `docs/`, deployed to GitHub Pages (D83)
- ✓ **CLI reference** — Auto-generated from clap help output (D84)

## Milestone 7: Polish, Identity & Platform Experience

> Brand identity, liveness monitoring, UX revamp, activity log, theme system, privacy communication, documentation overhaul. PRD #41.

### Wave 1 — Foundations (no dependencies, can start immediately) ✓

- [x] **SVG icon set** — Replace emoji/Unicode icons with consistent monochrome SVG set (#42)
- [x] **Native folder picker** — Tauri dialog.open() for Scan Roots + drag-and-drop on Settings (#43)
- [x] **Tooltips** — Add tooltips to all status indicators and domain terms (#44)
- [x] **Pagination** — User-configurable pagination (10/25/50/100) for Dashboard and Changes tables (#45)
- [x] **Unregister Repository** — Remove from tracking without touching disk; confirmation dialog (#46)
- [x] **Activity Log** — Operation history with filterable sidebar page, ring buffer storage (#47)
- [x] **Liveness core pipeline** — Model, probing, scheduler integration, dashboard dots (#48)
- [x] **Theme infrastructure** — Design-token override system + Default theme extraction (#49)
- [x] **Privacy communication** — In-app About section + docs Privacy page (#50)
- [x] **Mascot artwork** — Generate golden lion tamarin illustration + export icon assets (#51)

### Wave 2 — Build on Wave 1 ✓

- [x] **Top bar redesign** — Health indicator, global search, SVG notification bell (#52) ← #42
- [x] **Onboarding card** — First-run guidance on empty Dashboard (#53) ← #43
- [x] **Liveness endpoint discovery** — Convention-based scanning of docker-compose, Dockerfile, .env, k8s (#54) ← #48
- [x] **Liveness notifications** — Optional alerts on probe failure (#55) ← #48
- [x] **Dark theme** — Full design-token override (#56) ← #49
- [x] **World Cup - Brasil theme** — Full design-token override (#57) ← #49
- [x] **Theme switcher UI** — Settings preview cards + bottom bar toggle (#58) ← #49

### Wave 3 — Integration ✓

- [x] **Mascot per-theme variants** — Color-shifted variants across all touchpoints (#59) ← #51, #49
- [x] **Documentation overhaul** — Bruno-quality MkDocs Material theme + expanded content (#60)

### Wave 4 — Final

- [ ] **Final documentation update** — README, CONTEXT.md, DESIGN.md, all docs (#61) ← #42–#60
