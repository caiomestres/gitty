# Roadmap

## Milestone 1: Foundation

> Core infrastructure — Config, Scan Roots, Repository discovery. No UI yet.

- **Config system** — Read/write Config file, platform-aware paths, schema
- **Scan Root management** — Add/remove Scan Roots, recursive `.git` discovery
- **Repository registry** — UUID assignment, path tracking, persistence
- **Lock mechanism** — File-level locks preventing concurrent operations
- **Git read layer** — git2-based status, branch info, log, ref inspection
- **Git write layer** — Shell-out execution for pull, fetch, checkout, etc.

## Milestone 2: CLI

> Standalone CLI binary exposing core operations via `clap`.

- **CLI scaffold** — `gitty-cli` crate with clap, subcommands
- **Workspace commands** — `gitty scan`, `gitty list`, `gitty status`
- **Operation commands** — `gitty fetch`, `gitty pull`, `gitty checkout`
- **Group/Tag commands** — `gitty group`, `gitty tag`, `gitty filter`
- **Macro commands** — `gitty macro run`, `gitty macro define`

## Milestone 3: Desktop Shell

> Tauri app with navigation, layout, and Workspace dashboard.

- **App shell** — Sidebar navigation, main content area, status bar
- **Workspace dashboard** — Repository list with health indicators
- **Repository detail** — Branch info, status, recent commits
- **Scan Root management UI** — Add/remove/rescan

## Milestone 4: Organization & Operations

> Groups, Tags, Macros (with scripting), and bulk operations in both CLI and GUI.

- **Group management** — Create, rename, nest (arbitrary depth), assign Repositories
- **Tag management** — Create, assign, filter by Tags
- **Macro builder** — Define Steps (Git Operations + Shell Commands)
- **Macro scripting** — Variables (substitution), conditions (step skipping), rollback (undo on failure), confirmations (interactive prompts)
- **Macro execution** — Run Macros against selections, Job tracking
- **Job monitoring** — Progress, status, error reporting

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
