# State

## Decisions

| ID | Decision | Context | Date |
|----|----------|---------|------|
| D1 | Hybrid git execution (git2 reads, shell-out writes) | ADR-0001 — compatibility vs. performance trade-off | Pre-existing |
| D2 | Cargo workspace: gitty-core, gitty-cli, gitty-tauri | ADR-0002 — shared logic, independent binaries | Pre-existing |
| D3 | English only for all code and documentation | ADR-0003 | Pre-existing |
| D4 | Cursor-inspired design system from DESIGN.md | Warm cream canvas, editorial typography, hairline depth | Pre-existing |
| D5 | Foundation (Milestone 1) decomposed into 3 vertical-slice features: '.rf `foundation-discovery`, `foundation-git-write`, `foundation-lock` | Milestone too large to build/verify in one pass | 2026-06-01 |
| D6 | Config = single JSON file with schema `version`, no migrations during 0.x, hard-error on mismatch | ADR-0004 | 2026-06-01 |
| D7 | Repository identity = root-commit fingerprint; collision-safe auto re-link; `missing` state for vanished paths | ADR-0005 | 2026-06-01 |
| D8 | Scan = walkdir, descend into nested repos, never into `.git`, default ignore list, no symlinks, non-bare repos only | Grilling — balances nested discovery vs. <5s scan goal | 2026-06-01 |
| D9 | git2 (vendored libgit2) for reads; status exposes branch/detached/dirty/ahead-behind/upstream + HEAD commit summary + changed files | ADR-0001; E2 expansion for dashboard | 2026-06-01 |
| D10 | Git write = shell-out runner `(path, args) -> {code, stdout, stderr}`; git located via PATH, validated at startup (`foundation-git-write` feature) | ADR-0001 | 2026-06-01 |
| D11 | Lock = per-repo PID+timestamp lock file in config dir, stale detection, fail-fast on contention (`foundation-lock` feature) | ADR-0006 | 2026-06-01 |
| D12 | Errors = `thiserror` in gitty-core, `anyhow` at CLI boundary | Idiomatic Rust, tiny deps | 2026-06-01 |
| D13 | Git write runner = `std::process::Command`, no `which` crate; `GIT_TERMINAL_PROMPT=0` + SSH `BatchMode=yes` to prevent interactive prompts | ADR-0001; simplest safe approach | 2026-06-02 |
| D14 | Error classification = substring matching on stderr (Network, Conflict, Auth, BranchNotFound, DirtyWorkTree, NoUpstream, Unknown); no retry in Foundation | Actionable categories for CLI/GUI without Macro complexity | 2026-06-02 |
| D15 | Batch execution = sequential in v1; parallel requires Lock feature | Lock not yet built; sequential is correct and simple | 2026-06-02 |
| D16 | Foundation git write scope = fetch, pull, checkout; other operations added incrementally | Covers the primary bulk use cases; rebase/merge/stash deferred | 2026-06-02 |
| D17 | No daemon/IPC between CLI and GUI; both are independent processes sharing the same Config file with file-level locks | Simpler than GitKraken's cloud-mediated sync; fully local, no account needed | 2026-06-02 |
| D18 | Group = hierarchical tree (arbitrary nesting), Ungrouped as default; Tag = flat label, Favorite built-in; Macro = named step sequence with variables/conditions/rollback | CONTEXT.md domain model | 2026-06-02 |
| D19 | Config schema stays at v1; new fields (groups, macros) use `#[serde(default)]` for backward compat | No migration needed during 0.x | 2026-06-02 |
| D20 | Macro step parsing in CLI = `fetch`, `pull`, `checkout:<branch>`, `shell:<cmd>` | Simple inline syntax; JSON import deferred | 2026-06-02 |
| D21 | Tauri commands use stateless config access (load/save per call under Mutex); no managed state | Correct for v1; state management can optimize later | 2026-06-02 |
| D22 | Frontend design system = CSS custom properties from DESIGN.md; Inter substitutes for CursorGothic | Open-source font fallback | 2026-06-02 |
| D23 | M4 is incomplete — core + CLI done, GUI portion (Groups/Tags/Macros in Tauri + frontend) is outstanding | Grilling audit: ROADMAP marked ✓ but GUI has zero M4 functionality | 2026-06-02 |
| D24 | Remove `/repos` sidebar link (redundant with Dashboard); `/groups` stays for M4 GUI | Grilling Q2 | 2026-06-02 |
| D25 | Full Primary Workflow retroactively for M2/M3 audit + forward for M4 GUI | Grilling Q3 — no specs/PRDs/issues existed for M2/M3/M4 | 2026-06-02 |
| D26 | Sidebar nav: Dashboard, Groups, Macros, Settings | Grilling Q4 | 2026-06-02 |
| D27 | Sidebar IS the Group tree with Repositories listed under each Group (VS Code explorer style) | Grilling Q5+Q17 — sidebar is live navigation, Groups page is CRUD admin | 2026-06-02 |
| D28 | Groups page = CRUD-only admin panel (create/rename/delete/move Groups, no tree duplication) | Grilling Q6+Q17 | 2026-06-02 |
| D29 | Group assignment = dropdown on Repository detail page, shows "Moving from X to Y" | Grilling Q7+Q8 | 2026-06-02 |
| D30 | Tag management = inline editable list on Repository detail page | Grilling Q9 | 2026-06-02 |
| D31 | Dashboard gets a Tag filter dropdown above the repo table | Grilling Q10 | 2026-06-02 |
| D32 | Full visual Macro builder: step editor with Git Op picker, Shell Command input, conditions, rollback, variables | Grilling Q11+Q19 — user accepted scope | 2026-06-02 |
| D33 | Macro execution uses a Selection dialog (All/Group/Tag/individual Repos) | Grilling Q12 | 2026-06-02 |
| D34 | Job monitoring = inline progress/results panel during Macro execution | Grilling Q13 | 2026-06-02 |
| D35 | New Tauri commands as individual `#[tauri::command]` functions (same pattern as existing) | Grilling Q14 | 2026-06-02 |
| D36 | Managed state (`Mutex<Config>`) + `notify` file-watcher for config cache invalidation | ADR-0007 — Grilling Q15+Q18; supersedes D21 (stateless) | 2026-06-02 |
| D37 | Structured error DTOs with error codes, bundled into M4 GUI refactor (all commands) | Grilling Q16+Q20 | 2026-06-02 |
| D38 | 4 Health Checks via `trait HealthCheck`: Stale, Diverged, Dirty, Detached | Extensibility goal; future user-defined checks | 2026-06-02 |
| D39 | Stale = HEAD commit author-date age > configurable threshold (default 7d) | Uses existing CommitSummary.date | 2026-06-02 |
| D40 | Health thresholds configurable per-Workspace in Config with sensible defaults | stale:7d, diverged: >5=warn >20=crit | 2026-06-02 |
| D41 | Workspace Health score = (repos_not_critical / total_active) * 100; Missing repos excluded | CONTEXT.md definition | 2026-06-02 |
| D42 | Health evaluated: on-demand + background poll (configurable interval) + post-scheduler-run; writes to health.json | File lock for concurrent access; atomic temp+rename | 2026-06-02 |
| D43 | health.json as separate cache file; CLI reads cached data with "last evaluated" timestamp | Not in Config (computed state) | 2026-06-02 |
| D44 | Change Dashboard = git2 Revwalk on HEAD default; "show all branches" toggle per repo; in-memory cache invalidated on fetch/pull | | 2026-06-02 |
| D45 | Scheduler runs in GUI (tokio task) and CLI (self-daemonizing); lock ensures single instance; PID file | ADR-0008 | 2026-06-02 |
| D46 | Scheduler triggers: simple (interval) + advanced (interval + time window + day constraints) | Stored in Config | 2026-06-02 |
| D47 | Power-state aware scheduling: battery/sysinfo crate; battery-level threshold + AC-only toggle; graceful degradation | | 2026-06-02 |
| D48 | Scheduler default action = system Macro `__scheduler_default` (fetch all); user can reassign | | 2026-06-02 |
| D49 | Scheduler state (last_run, next_run) persisted in Config | | 2026-06-02 |
| D50 | Notifications: OS-native toasts (tauri-plugin-notification) for critical + in-app panel for all | | 2026-06-02 |
| D51 | Notification triggers configurable in Config; user picks severity threshold | Configurable from CLI and GUI | 2026-06-02 |
| D52 | Notification dedup: aggregate ("3 repos critical" as single notification) | | 2026-06-02 |
| D53 | Notifications stored in Config as bounded Vec with 7-day TTL auto-purge on load | Not a separate file; small volume | 2026-06-02 |
| D54 | No Config schema bump for M5; #[serde(default)] for all new fields | D19 continues | 2026-06-02 |
| D55 | M5 delivers core + CLI + GUI together (lesson from D23) | End-to-end features | 2026-06-02 |
| D56 | CLI: `gitty health` (all repos + score), `gitty health --repo <id>` (drill-down) | | 2026-06-02 |
| D57 | GUI scheduler = tokio task in setup(), loops 30s, checks should_run(), executes Macro, re-evaluates health, generates notifications | M5 Grilling Q1 | 2026-06-02 |
| D58 | CLI scheduler = full fork/detach daemonization: `daemonize` crate (Unix), `DETACHED_PROCESS` (Windows) | M5 Grilling Q2+Q16; cross-platform | 2026-06-02 |
| D59 | compute_next_run for Advanced mode respects window/day constraints; scans forward until valid slot (7-day cap) | M5 Grilling Q3 | 2026-06-02 |
| D60 | Midnight-crossing time windows supported: if start > end, window wraps (current >= start OR current <= end) | M5 Grilling Q4 | 2026-06-02 |
| D61 | SchedulerConfig uses proper types: OffsetDateTime for timestamps, Uuid for macro_id, NaiveTime/Weekday for Advanced trigger | M5 Grilling Q5 | 2026-06-02 |
| D62 | Notification.timestamp uses OffsetDateTime, not String | M5 Grilling Q6 | 2026-06-02 |
| D63 | HealthCheck::evaluate trait accepts `now: OffsetDateTime` parameter for deterministic testing | M5 Grilling Q7 | 2026-06-02 |
| D64 | Changes page uses onMount (not $effect) for initial load; handlers drive subsequent loads | M5 Grilling Q8 | 2026-06-02 |
| D65 | Per-repo "show all branches" toggle icon on Changes page | M5 Grilling Q9; GUI-CHANGE-04 | 2026-06-02 |
| D66 | Settings scheduler section saves on change via set_scheduler_config (matches notification pattern) | M5 Grilling Q10 | 2026-06-02 |
| D67 | Repository.display_name() method replaces 3 duplicate free functions | M5 Grilling Q11 | 2026-06-02 |
| D68 | evaluate_workspace accepts HashMap<Uuid, RepositoryStatus> for O(1) lookup | M5 Grilling Q12 | 2026-06-02 |
| D69 | Severity DTOs use serde serialization directly; manual severity_str() helpers removed | M5 Grilling Q13 | 2026-06-02 |
| D70 | M5 complete = every acceptance criterion passes including battery detection and CLI daemonization | M5 Grilling Q14 | 2026-06-02 |
| D71 | battery crate integrated for real hardware battery state detection in scheduler loop | M5 Grilling Q14+Q15 resolved | 2026-06-02 |
| D72 | M6 split into 3 sub-features: `m6-gui-completion` (M4+M5 GUI), `m6-polish` (design system + errors + UX), `m6-ship` (packaging + docs + signing) | Scope too large for single spec; each gets own spec/design/tasks | 2026-06-06 |
| D73 | Design system = token-complete + component audit; every DESIGN.md token as CSS custom property + systematic audit of all Svelte components for hardcoded values | M6 Grilling Q1 | 2026-06-06 |
| D74 | Self-host fonts: bundle Inter + JetBrains Mono in `static/fonts/`, `@font-face` declarations; remove Google Fonts CDN import; CSP-clean, fully offline | M6 Grilling Q2; current CDN import silently blocked by CSP | 2026-06-06 |
| D75 | Skip timeline pastel colors from DESIGN.md (no agent timeline in Gitty) | M6 Grilling Q3 | 2026-06-06 |
| D76 | Macro-level retry: optional `retry` config on Step; Git Ops only, Network errors only; default max 3 attempts with exponential backoff | M6 Grilling Q4+Q19; CONTEXT.md says Shell Commands never auto-retried | 2026-06-06 |
| D77 | Toast + contextual error display: transient errors (network, lock) as auto-dismissing toasts; persistent errors (config, git not found) inline with recovery suggestions | M6 Grilling Q5 | 2026-06-06 |
| D78 | AppError DTO gets `hint` field populated from CoreError variant metadata; frontend displays recovery hints | M6 Grilling Q6 | 2026-06-06 |
| D79 | GitHub Actions release matrix: Windows NSIS, macOS DMG, Linux AppImage; triggered on version tags | M6 Grilling Q7+Q8 | 2026-06-06 |
| D80 | Auto-generated release notes via git-cliff from conventional commits | M6 Grilling Q9 | 2026-06-06 |
| D81 | Windows code signing via SignPath.io (free for OSS) integrated into CI release workflow; M6 scope | M6 Grilling Q10; moved from Deferred Ideas | 2026-06-06 |
| D82 | macOS: ad-hoc codesign in CI + Gatekeeper bypass docs + Homebrew cask tap; no Apple Developer account ($99/yr) needed | ADR-0009; M6 Grilling Q11 | 2026-06-06 |
| D83 | Full docs site via MkDocs Material, source in `docs/`, deployed to GitHub Pages | M6 Grilling Q12+Q13 | 2026-06-06 |
| D84 | CLI reference auto-generated from clap help output; stays in sync automatically | M6 Grilling Q14 | 2026-06-06 |
| D85 | M6 absorbs outstanding M4 GUI work (sidebar group tree, groups admin, macro builder, macro execution, repo detail enhancements) | M6 Grilling Q15; D23 acknowledged M4 GUI was outstanding | 2026-06-06 |
| D86 | M6 includes M5 GUI polish (health dashboard, changes page, scheduler settings, notification panel) | M6 Grilling Q16 | 2026-06-06 |

## Blockers

_None currently._

## Lessons

- git2 0.21 accessors return `Result` (not `Option`) for `Reference::shorthand`/`name`, `Commit::summary`, `Buf::as_str`. Check the registry source when unsure.
- `git2` with `default-features = false` builds vendored libgit2 on Windows in ~46s with no cmake/OpenSSL — reads are local-only, so the `ssh`/`https` features are unnecessary.
- The CLI binary is named `gitty` via an explicit `[[bin]]` in `gitty-cli` (package name would otherwise be `gitty-cli`).
- GitKraken CLI (Go) and Desktop (Electron) sync via cloud account, not local IPC. Our shared `gitty-core` crate approach is simpler for a local-first tool.
- Svelte 5 runes (`$state`, `$derived`, `$effect`) with Tauri `invoke()` provide a clean data-flow model without stores.
- Tauri 2 `#[tauri::command]` functions returning `Result<T, String>` is the simplest IPC pattern; typed DTO structs keep the frontend/backend contract explicit.
- **Windows Smart App Control (SAC) blocks all unsigned executables**, including Rust build scripts. Developers must disable SAC to build from source. For end-user distribution, release binaries must be code-signed (SignPath.io, free for OSS). This is a pre-release blocker — tracked in Deferred Ideas.
- **CRITICAL (2026-06-02): Agent skipped the entire Primary Workflow (grill → PRD → spec → design → tasks → issues → execute → review → close-out) for M2/M3/M4 and delegated to weaker-model subagents. Code produced without the workflow must be fully reviewed. Added hard-stop rules to AGENTS.md to prevent recurrence. ALWAYS follow the workflow. ALWAYS match the parent model tier for subagents.**

## Deferred Ideas

| Idea | Reason | Source |
|------|--------|--------|
| Dependency Map | Complexity; core features work without it | CONTEXT.md — explicitly v2 |
| GitHub/GitLab API integration | Out of v1 scope | PROJECT.md |
| Manual re-link resolution UI (for ambiguous fingerprint matches) | `foundation-discovery` auto-links only unambiguous matches | ADR-0005 |
| Submodule-aware discovery (flag/filter submodules) | `foundation-discovery` treats nested repos generically | Grilling D1 |
| ~~Time-windowed Change Dashboard~~ | **Active scope — M5** | ROADMAP.md |
| Macro JSON import/export | CLI defines steps inline; deferred file-based definition | M2/M4 |
| ~~Tauri managed state~~ | **Decided (D36):** Managed state + file-watcher in ADR-0007 | M3 → Grilling |
| Native file picker for Scan Root dialog | Using text input for now; `tauri-plugin-dialog` deferred | M3 |
| ~~Windows code signing via SignPath.io~~ | **Active scope — M6 (D81)** | M5 Grilling → M6 |
| macOS Apple Developer notarization ($99/yr) | Using free ad-hoc codesign + Homebrew cask for v1 (D82, ADR-0009); full notarization when budget allows | M6 Grilling |

## Preferences

- Lightweight tasks (validation, state updates) work well with faster/cheaper models.

## Todos

- [x] Set up Cargo workspace structure (3 crates per ADR-0002)
- [x] Add `git2`, `dirs`, `walkdir`, `thiserror` deps to gitty-core; `anyhow` to gitty-cli (+ `time`, `dunce`)
- [x] Verify `git2` (vendored libgit2) builds on Windows before committing
- [x] `foundation-discovery` feature — config, repository, scan, git::read, re-link, CLI (scan/list/status) — all 12 DISC reqs verified
- [x] `foundation-git-write` feature — git write layer (shell-out runner, fetch/pull/checkout) — all 11 GWRITE reqs verified
- [x] `foundation-lock` feature — per-repo PID lock files (ADR-0006), stale detection, fail-fast on contention, integrated into batch execution
- [x] M4: Group, Tag, Macro domain models in gitty-core (6 new modules, 21 new tests)
- [x] M3: Tauri command bridge — 10 IPC commands (list, status, scan, remove, fetch, pull, checkout, fetch_all, pull_all)
- [x] M3: Frontend design system — CSS tokens from DESIGN.md, global reset, Inter/JetBrains Mono
- [x] M3: App shell — sidebar nav, status bar, bottom bar, main content area
- [x] M3: Workspace dashboard — stats cards, repo table with status, scan dialog, fetch all
- [x] M3: Repository detail view — branch/status/tracking/commit info, fetch/pull actions
- [x] M3: Settings page — Scan Root CRUD (add/remove/rescan)
- [x] M2: CLI group commands — list, create, rename, delete, assign, tree
- [x] M2: CLI tag commands — list, add, remove
- [x] M2: CLI filter command — filter by --group and/or --tag
- [x] M2: CLI macro commands — list, define, delete, show, run (with --group/--tag/--repo selectors)
- [x] M6: Grilling complete — 16 decisions recorded (D72–D86), ADR-0009 created
- [x] M6: PRD updated on GitHub Issues (#1) with M6 scope + user stories #31-37
- [x] M6: `m6-gui-completion` — specified (spec + tasks, 31 reqs, 10 tasks, issues #18-24)
- [x] M6: `m6-polish` — specified (spec + design + tasks, 30 reqs, 10 tasks, issues #25-29)
- [x] M6: `m6-ship` — specified (spec + tasks, 25 reqs, 9 tasks, issues #30-33)
- [x] M6: Execute `m6-gui-completion` (#18-24)
- [x] M6: Execute `m6-polish` (#25-29)
- [x] M6: Execute `m6-ship` (#30-33)
- [ ] M6: Thermo-nuclear code quality review
- [ ] M6: Close-out
