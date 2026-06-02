# State

## Decisions

| ID | Decision | Context | Date |
|----|----------|---------|------|
| D1 | Hybrid git execution (git2 reads, shell-out writes) | ADR-0001 — compatibility vs. performance trade-off | Pre-existing |
| D2 | Cargo workspace: gitty-core, gitty-cli, gitty-tauri | ADR-0002 — shared logic, independent binaries | Pre-existing |
| D3 | English only for all code and documentation | ADR-0003 | Pre-existing |
| D4 | Cursor-inspired design system from DESIGN.md | Warm cream canvas, editorial typography, hairline depth | Pre-existing |
| D5 | Foundation (Milestone 1) decomposed into 3 vertical-slice features: `foundation-discovery`, `foundation-git-write`, `foundation-lock` | Milestone too large to build/verify in one pass | 2026-06-01 |
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

## Blockers

_None currently._

## Lessons

- git2 0.21 accessors return `Result` (not `Option`) for `Reference::shorthand`/`name`, `Commit::summary`, `Buf::as_str`. Check the registry source when unsure.
- `git2` with `default-features = false` builds vendored libgit2 on Windows in ~46s with no cmake/OpenSSL — reads are local-only, so the `ssh`/`https` features are unnecessary.
- The CLI binary is named `gitty` via an explicit `[[bin]]` in `gitty-cli` (package name would otherwise be `gitty-cli`).
- GitKraken CLI (Go) and Desktop (Electron) sync via cloud account, not local IPC. Our shared `gitty-core` crate approach is simpler for a local-first tool.
- Svelte 5 runes (`$state`, `$derived`, `$effect`) with Tauri `invoke()` provide a clean data-flow model without stores.
- Tauri 2 `#[tauri::command]` functions returning `Result<T, String>` is the simplest IPC pattern; typed DTO structs keep the frontend/backend contract explicit.
- **CRITICAL (2026-06-02): Agent skipped the entire Primary Workflow (grill → PRD → spec → design → tasks → issues → execute → review → close-out) for M2/M3/M4 and delegated to weaker-model subagents. Code produced without the workflow must be fully reviewed. Added hard-stop rules to AGENTS.md to prevent recurrence. ALWAYS follow the workflow. ALWAYS match the parent model tier for subagents.**

## Deferred Ideas

| Idea | Reason | Source |
|------|--------|--------|
| Dependency Map | Complexity; core features work without it | CONTEXT.md — explicitly v2 |
| GitHub/GitLab API integration | Out of v1 scope | PROJECT.md |
| Manual re-link resolution UI (for ambiguous fingerprint matches) | `foundation-discovery` auto-links only unambiguous matches | ADR-0005 |
| Submodule-aware discovery (flag/filter submodules) | `foundation-discovery` treats nested repos generically | Grilling D1 |
| Time-windowed Change Dashboard (24h/7d/30d, group by author/repo/branch) | Aggregate history — Milestone 5 | ROADMAP.md |
| Macro JSON import/export | CLI defines steps inline; deferred file-based definition | M2/M4 |
| ~~Tauri managed state~~ | **Decided (D36):** Managed state + file-watcher in ADR-0007 | M3 → Grilling |
| Native file picker for Scan Root dialog | Using text input for now; `tauri-plugin-dialog` deferred | M3 |

## Preferences

_None recorded yet._

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
