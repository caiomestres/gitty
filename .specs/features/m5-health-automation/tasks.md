# M5: Health, Dashboard & Automation — Tasks

**Design**: `.specs/features/m5-health-automation/design.md`
**Status**: Draft

---

## Execution Plan

### Phase 1: Foundation (Sequential)

Data models, Config extensions, and the health trait infrastructure.

```
T1 → T2 → T3
```

### Phase 2: Core Engines (Parallel)

Independent domain modules that build on the foundation.

```
       ┌→ T4 (health checks) ─────┐
T3 ──→ ├→ T5 (health cache) ──────┼──→ T10
       ├→ T6 (changes module) ─────┤
       └→ T7 (scheduler core) ────┘
              │
              └→ T8 (scheduler daemon)
                    │
                    └→ T9 (notification module)
```

### Phase 3: CLI Commands (Sequential after Phase 2)

```
T10 → T11 → T12
```

### Phase 4: Tauri IPC (Parallel after Phase 3)

```
        ┌→ T13 (health commands) ──┐
T12 ──→ ├→ T14 (changes commands) ─┼──→ T17
        ├→ T15 (scheduler commands)┤
        └→ T16 (notification cmds) ┘
```

### Phase 5: Frontend (Parallel after Phase 4)

```
        ┌→ T18 (health dashboard) ─┐
T17 ──→ ├→ T19 (changes dashboard) ┼──→ T22
        ├→ T20 (notification panel) ┤
        └→ T21 (settings extension) ┘
```

### Phase 6: Integration (Sequential)

```
T22 → T23 → T24
```

---

## Task Breakdown

### T1: Health Check trait + data models

**What**: Define `HealthCheck` trait, `CheckSeverity`, `CheckResult`, `RepositoryHealth`, `WorkspaceHealth`, `HealthThresholds` structs in a new `health` module.
**Where**: `crates/gitty-core/src/health.rs`
**Depends on**: None
**Reuses**: `repository::Repository`, `git::read::RepositoryStatus`
**Requirement**: HEALTH-01

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `trait HealthCheck` defined with `fn id(&self) -> &str` and `fn evaluate(&self, status: &RepositoryStatus, thresholds: &HealthThresholds) -> CheckResult`
- [ ] All data model structs defined with Serialize/Deserialize
- [ ] `HealthThresholds` has `Default` impl with documented defaults (stale: 7/14 days, diverged: 5/20)
- [ ] Module re-exported from `lib.rs`
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

**Tests**: unit
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T2: Config extensions for M5

**What**: Add `health_thresholds`, `scheduler`, `notifications`, `notification_history` fields to Config/Workspace with `#[serde(default)]`.
**Where**: `crates/gitty-core/src/repository.rs` (Workspace), `crates/gitty-core/src/config/mod.rs` (Config)
**Depends on**: T1
**Reuses**: Existing `#[serde(default)]` pattern from Workspace fields
**Requirement**: CACHE-01, SCHED-01, NOTIF-CFG-02

**Tools**:
- Skill: none

**Done when**:
- [ ] `Workspace` gains `health_thresholds: HealthThresholds` (serde default)
- [ ] `Config` gains `scheduler: SchedulerConfig` (serde default)
- [ ] `Config` gains `notifications: NotificationConfig` (serde default)
- [ ] `Config` gains `notification_history: Vec<Notification>` (serde default)
- [ ] Existing tests still pass (round-trip, schema version)
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

**Tests**: unit (existing Config tests verify backward compat)
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T3: Implement 4 Health Check structs

**What**: Implement `StaleCheck`, `DivergedCheck`, `DirtyCheck`, `DetachedCheck` — each implementing `HealthCheck` trait.
**Where**: `crates/gitty-core/src/health.rs`
**Depends on**: T1
**Reuses**: `git::read::RepositoryStatus` fields directly
**Requirement**: HEALTH-02, HEALTH-03, HEALTH-04, HEALTH-05, HEALTH-06, HEALTH-07

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `StaleCheck` compares HEAD date to threshold; warning if > stale_days_warning, critical if > stale_days_critical
- [ ] `DivergedCheck` checks `upstream.behind`; skips if no upstream (HEALTH-07); warning/critical per thresholds
- [ ] `DirtyCheck` returns warning if `dirty == true`
- [ ] `DetachedCheck` returns critical if `detached == true`
- [ ] `evaluate_repository()` function runs all checks, skips Missing repos (HEALTH-06), returns `RepositoryHealth`
- [ ] `evaluate_workspace()` computes score as `(not_critical / total_active) * 100` (SCORE-01 through SCORE-04)
- [ ] Tests cover: each check individually, missing repo skip, no-upstream skip, score calculation, zero-repos edge case, empty-repo edge case
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥12 new tests

**Tests**: unit
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T4: Health cache module [P]

**What**: Implement `health_cache` module — save/load `WorkspaceHealth` to/from `health.json` with atomic writes and file locking.
**Where**: `crates/gitty-core/src/health_cache.rs`
**Depends on**: T3
**Reuses**: Config's atomic temp+rename pattern, `config::paths::config_dir()`
**Requirement**: CACHE-01, CACHE-02, CACHE-03, CACHE-04, CACHE-05

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `save()` writes `CachedHealth` (with `last_evaluated` timestamp) to `health.json` via temp+rename
- [ ] `save()` acquires advisory file lock (via `fs2`) before writing
- [ ] `load()` returns `None` if file missing, parses JSON if present
- [ ] `load()` returns `None` (discards) if file is corrupt/malformed
- [ ] `CachedHealth` struct includes `last_evaluated: String` (RFC3339) and full `WorkspaceHealth`
- [ ] Tests: save round-trip, missing file, corrupt file, concurrent access simulation
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥6 new tests

**Tests**: unit + integration (file I/O)
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T5: Changes module [P]

**What**: Implement `changes` module — scan commit history via `git2::Revwalk`, filter by time window, group results.
**Where**: `crates/gitty-core/src/changes.rs`
**Depends on**: T3 (uses Repository type, but no health dependency — parallel with T4)
**Reuses**: `git2::Repository::open()` pattern from `git::read`
**Requirement**: CHANGE-01 through CHANGE-08

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `ChangeEntry` struct defined (commit_hash, author, date, subject, branch, repo_id, repo_name)
- [ ] `TimeWindow` enum (Day, Week, Month) with `fn cutoff(&self) -> OffsetDateTime`
- [ ] `Grouping` enum (Author, Repository, Branch)
- [ ] `scan_changes()` walks HEAD commits for each repo, filtering by time window
- [ ] `scan_changes()` supports `all_branches` set — expands to all local branches with upstream for specified repos
- [ ] `group_changes()` returns `BTreeMap<String, Vec<&ChangeEntry>>` grouped by selected dimension
- [ ] Edge cases: empty repo returns empty, shallow clone returns available commits, zero results returns empty Vec
- [ ] Tests with real git repos in tempdir: create commits at known dates, verify filtering and grouping
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥8 new tests

**Tests**: integration (requires real git repos)
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T6: Scheduler core module [P]

**What**: Implement scheduler trigger logic — `SchedulerConfig`, `SchedulerTrigger`, `PowerConfig`, `should_run()`, `record_run()`, `compute_next_run()`.
**Where**: `crates/gitty-core/src/scheduler.rs`
**Depends on**: T2 (needs SchedulerConfig in Config)
**Reuses**: `time` crate for date math
**Requirement**: SCHED-01, SCHED-02, SCHED-03, TRIG-01 through TRIG-06

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `SchedulerConfig`, `SchedulerTrigger`, `PowerConfig` structs with Serialize/Deserialize and Default impls
- [ ] `should_run(config, now, on_battery, battery_level) -> bool` — pure function, no side effects
- [ ] Simple trigger: returns true if `now - last_run >= interval`
- [ ] Advanced trigger: returns true if within time window AND on valid day AND interval elapsed
- [ ] Power check: returns false if `on_battery && battery_level < threshold`
- [ ] `record_run()` updates `last_run` and computes `next_run`
- [ ] `compute_next_run()` returns next valid run time respecting window/day constraints
- [ ] Tests: simple interval, advanced window (inside/outside), day filtering, power pause/resume, hot reload (config change)
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥10 new tests

**Tests**: unit (pure functions with fake clock via parameter injection)
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T7: Scheduler daemon module

**What**: Implement self-daemonizing process lifecycle — `start()`, `stop()`, `status()` with PID file management.
**Where**: `crates/gitty-core/src/scheduler/daemon.rs` (split scheduler into a directory module)
**Depends on**: T6
**Reuses**: Lock module's PID file pattern
**Requirement**: SCHED-05, SCHED-06, SCHED-07, SCHED-08

**Tools**:
- Skill: none

**Done when**:
- [ ] `start()` forks/detaches (Unix: `fork`+`setsid` via `daemonize` crate; Windows: `DETACHED_PROCESS`)
- [ ] `start()` writes PID file to Config directory
- [ ] `start()` refuses if PID file exists and process is alive (SCHED-06)
- [ ] `stop()` reads PID file, sends SIGTERM (Unix) or TerminateProcess (Windows)
- [ ] `status()` returns `SchedulerStatus { running, pid, last_run, next_run }`
- [ ] Stale PID detection: if PID file exists but process dead, delete and allow start
- [ ] Tests: PID file write/read/delete, stale detection (can't easily test fork in unit tests — integration test with actual subprocess)
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥4 new tests

**Tests**: unit + integration
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T8: Notification module

**What**: Implement notification generation, storage, and TTL purge logic.
**Where**: `crates/gitty-core/src/notification.rs`
**Depends on**: T3 (needs WorkspaceHealth for comparison)
**Reuses**: None (new domain)
**Requirement**: NOTIF-01 through NOTIF-08, NOTIF-CFG-01 through NOTIF-CFG-04

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] `Notification`, `Severity`, `NotificationTrigger`, `NotificationConfig` structs with Serialize/Deserialize
- [ ] `generate_health_notification(prev, current, trigger) -> Option<Notification>` — aggregate logic
- [ ] OnCritical: only fires when repos transition to critical
- [ ] OnAnyChange: fires on any severity change
- [ ] OnSchedulerComplete: fires after scheduler run (always, if enabled)
- [ ] Disabled: returns None always
- [ ] Aggregate message: "N repos are critical" (not individual per-repo)
- [ ] `purge_expired(notifications, ttl_days)` removes entries older than TTL
- [ ] Tests: each trigger mode, aggregate formatting, purge logic, empty prev state
- [ ] Gate passes: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥8 new tests

**Tests**: unit
**Gate**: `cargo test -p gitty-core && cargo clippy -p gitty-core -- -D warnings && cargo fmt --check`

---

### T9: Wire modules in lib.rs + add dependencies to Cargo.toml

**What**: Register new modules in `lib.rs`, add `fs2`, `battery`, `daemonize` (Unix) to `Cargo.toml`.
**Where**: `crates/gitty-core/src/lib.rs`, `crates/gitty-core/Cargo.toml`
**Depends on**: T4, T5, T6, T7, T8 (all modules exist)
**Reuses**: Existing module registration pattern
**Requirement**: All (wiring)

**Tools**:
- Skill: none

**Done when**:
- [ ] `pub mod health;`, `pub mod health_cache;`, `pub mod changes;`, `pub mod scheduler;`, `pub mod notification;` in lib.rs
- [ ] Appropriate `pub use` re-exports for key types
- [ ] `fs2`, `battery` added to `[dependencies]` in Cargo.toml
- [ ] `daemonize` added under `[target.'cfg(unix)'.dependencies]`
- [ ] Full workspace builds: `cargo build`
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: none (wiring only)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T10: CLI `gitty health` command

**What**: Add `health` subcommand to CLI — displays per-repo checks and aggregate score.
**Where**: `crates/gitty-cli/src/main.rs` (or split into submodule)
**Depends on**: T9
**Reuses**: Existing CLI command pattern (clap subcommands)
**Requirement**: CLI-01, CLI-02, CLI-03, CLI-04

**Tools**:
- Skill: none

**Done when**:
- [ ] `gitty health` displays Workspace Health score + per-repo table (name, worst severity, check details)
- [ ] `gitty health --repo <id-or-name>` displays single-repo drill-down
- [ ] If health.json cache exists, shows cached data with "last evaluated: <timestamp>"
- [ ] If no cache, performs fresh evaluation
- [ ] Output is human-readable table format
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit (output formatting)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T11: CLI `gitty scheduler` command

**What**: Add `scheduler` subcommand with `start`, `stop`, `status` sub-subcommands.
**Where**: `crates/gitty-cli/src/main.rs`
**Depends on**: T10
**Reuses**: Existing CLI command pattern
**Requirement**: SCHED-05, SCHED-07, SCHED-08

**Tools**:
- Skill: none

**Done when**:
- [ ] `gitty scheduler start` invokes daemon start; reports success or "already running"
- [ ] `gitty scheduler stop` invokes daemon stop; reports success or "not running"
- [ ] `gitty scheduler status` displays running/stopped, PID, last_run, next_run
- [ ] Proper error messages for all failure cases
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit (output formatting; daemon lifecycle tested in T7)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T12: Export gitty-core public API for Tauri

**What**: Ensure all types needed by Tauri IPC are public and serializable (DTO-ready). Add any missing `Serialize` derives.
**Where**: `crates/gitty-core/src/lib.rs` (re-exports)
**Depends on**: T10, T11
**Reuses**: Existing pub use pattern
**Requirement**: All GUI requirements

**Tools**:
- Skill: none

**Done when**:
- [ ] `WorkspaceHealth`, `RepositoryHealth`, `CheckResult`, `CheckSeverity`, `HealthThresholds` exported
- [ ] `ChangeEntry`, `TimeWindow`, `Grouping` exported
- [ ] `SchedulerConfig`, `SchedulerStatus`, `NotificationConfig`, `Notification` exported
- [ ] All exported types derive `Serialize` + `Deserialize`
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: none (wiring)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T13: Tauri health IPC commands [P]

**What**: Implement `get_workspace_health`, `get_repository_health`, `refresh_health` Tauri commands.
**Where**: `src-tauri/src/commands/health.rs`
**Depends on**: T12
**Reuses**: Existing Tauri command pattern from `src-tauri/src/commands/`
**Requirement**: GUI-HEALTH-01 through GUI-HEALTH-04

**Tools**:
- Skill: `tauri`

**Done when**:
- [ ] `get_workspace_health()` returns cached health or evaluates fresh if no cache
- [ ] `get_repository_health(repo_id)` returns single repo's check results
- [ ] `refresh_health()` forces fresh evaluation, updates cache, returns new results
- [ ] Commands registered in Tauri app builder
- [ ] Proper error DTOs for failure cases
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit (command logic)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T14: Tauri changes IPC commands [P]

**What**: Implement `get_changes` Tauri command.
**Where**: `src-tauri/src/commands/changes.rs`
**Depends on**: T12
**Reuses**: Existing Tauri command pattern
**Requirement**: GUI-CHANGE-01 through GUI-CHANGE-04

**Tools**:
- Skill: `tauri`

**Done when**:
- [ ] `get_changes(window, grouping, all_branches_repos)` returns grouped changes
- [ ] In-memory cache (`Mutex<Option<...>>`) for session-level caching
- [ ] Cache invalidated when fetch/pull commands complete (hook into existing commands)
- [ ] Command registered in Tauri app builder
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T15: Tauri scheduler IPC commands [P]

**What**: Implement `get_scheduler_status`, `set_scheduler_config` Tauri commands.
**Where**: `src-tauri/src/commands/scheduler.rs`
**Depends on**: T12
**Reuses**: Existing Tauri command pattern
**Requirement**: SCHED-04, TRIG-06

**Tools**:
- Skill: `tauri`

**Done when**:
- [ ] `get_scheduler_status()` returns current scheduler state
- [ ] `set_scheduler_config(config)` persists to Config and triggers immediate schedule recalculation
- [ ] GUI scheduler tokio task created (background loop checking `should_run` every 30s)
- [ ] Task defers if CLI daemon already running (checks PID file)
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T16: Tauri notification IPC commands [P]

**What**: Implement `get_notifications`, `mark_notification_read`, `get_notification_config`, `set_notification_config` commands.
**Where**: `src-tauri/src/commands/notifications.rs`
**Depends on**: T12
**Reuses**: Existing Tauri command pattern
**Requirement**: NOTIF-06, NOTIF-CFG-01 through NOTIF-CFG-04

**Tools**:
- Skill: `tauri`

**Done when**:
- [ ] `get_notifications()` returns notification history (purges expired first)
- [ ] `mark_notification_read(id)` updates notification read status
- [ ] `get_notification_config()` returns current config
- [ ] `set_notification_config(config)` persists immediately
- [ ] OS toast emission via `tauri-plugin-notification` when notification is generated with critical severity
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T17: Frontend TypeScript types + invoke helpers

**What**: Define TypeScript interfaces for all M5 DTOs and create invoke wrapper functions.
**Where**: `src/lib/types/health.ts`, `src/lib/types/changes.ts`, `src/lib/types/notifications.ts`
**Depends on**: T13, T14, T15, T16
**Reuses**: Existing `src/lib/types/workspace.ts` pattern
**Requirement**: All GUI requirements

**Tools**:
- Skill: none

**Done when**:
- [ ] All DTO interfaces defined matching Rust serialization
- [ ] `invoke` wrapper functions typed (e.g., `getWorkspaceHealth(): Promise<WorkspaceHealthDto>`)
- [ ] No TypeScript errors: `npm run check`

**Tests**: none (types only)
**Gate**: `npm run check`

---

### T18: Health Dashboard page [P]

**What**: Create `/health` route with Workspace Health score display and per-repo health table with drill-down.
**Where**: `src/routes/health/+page.svelte`
**Depends on**: T17
**Reuses**: Stats cards pattern from workspace dashboard, table component pattern, DESIGN.md tokens
**Requirement**: GUI-HEALTH-01 through GUI-HEALTH-05

**Tools**:
- Skill: `frontend-design`

**Done when**:
- [ ] Score displayed prominently (large number + percentage)
- [ ] Per-repo table with traffic-light severity indicators (green/yellow/red dots)
- [ ] Click repo row → drill-down showing individual check results
- [ ] "Refresh Health" button triggers fresh evaluation
- [ ] Loading state while evaluation runs
- [ ] Follows DESIGN.md (cream canvas, hairline depth, Inter font)
- [ ] Gate passes: `npm run check`

**Tests**: none (visual; Playwright later)
**Gate**: `npm run check`

---

### T19: Changes Dashboard page [P]

**What**: Create `/changes` route with time window selector, grouping controls, and commit list display.
**Where**: `src/routes/changes/+page.svelte`
**Depends on**: T17
**Reuses**: Table/list patterns from workspace dashboard, DESIGN.md tokens
**Requirement**: GUI-CHANGE-01 through GUI-CHANGE-04

**Tools**:
- Skill: `frontend-design`

**Done when**:
- [ ] Time window selector (24h / 7d / 30d) — default 7d
- [ ] Grouping mode selector (Author / Repository / Branch)
- [ ] Grouped commit list with author, date, subject, repo name
- [ ] "Show all branches" toggle per repo
- [ ] Empty state when no commits in window
- [ ] Follows DESIGN.md
- [ ] Gate passes: `npm run check`

**Tests**: none (visual)
**Gate**: `npm run check`

---

### T20: Notification panel component [P]

**What**: Create in-app notification panel (badge + dropdown) in the app shell.
**Where**: `src/lib/components/NotificationPanel.svelte`
**Depends on**: T17
**Reuses**: AppShell component, DESIGN.md tokens
**Requirement**: NOTIF-06

**Tools**:
- Skill: `frontend-design`

**Done when**:
- [ ] Bell icon in top bar with unread count badge
- [ ] Click opens dropdown panel showing notification list (title, body, timestamp, severity indicator)
- [ ] Clicking a notification marks it read
- [ ] Panel shows empty state when no notifications
- [ ] Follows DESIGN.md
- [ ] Gate passes: `npm run check`

**Tests**: none (visual)
**Gate**: `npm run check`

---

### T21: Settings page extensions [P]

**What**: Add Scheduler and Notification configuration sections to the Settings page.
**Where**: `src/routes/settings/+page.svelte`
**Depends on**: T17
**Reuses**: Existing settings page layout
**Requirement**: NOTIF-CFG-01, TRIG-01, TRIG-02, POLL-02

**Tools**:
- Skill: `frontend-design`

**Done when**:
- [ ] Scheduler section: enable/disable toggle, interval input, advanced mode (time window, day checkboxes), power settings
- [ ] Notifications section: trigger dropdown (critical/any-change/scheduler-complete/disabled), polling interval
- [ ] Changes save immediately on interaction (invoke set_* commands)
- [ ] Follows DESIGN.md
- [ ] Gate passes: `npm run check`

**Tests**: none (visual)
**Gate**: `npm run check`

---

### T22: Sidebar navigation updates

**What**: Add Health and Changes links to the sidebar navigation.
**Where**: `src/lib/components/Sidebar.svelte`
**Depends on**: T18, T19, T20, T21
**Reuses**: Existing sidebar link pattern
**Requirement**: GUI-HEALTH-01, GUI-CHANGE-01

**Tools**:
- Skill: none

**Done when**:
- [ ] "Health" link added to sidebar (navigates to /health)
- [ ] "Changes" link added to sidebar (navigates to /changes)
- [ ] Active state highlighting works correctly
- [ ] Gate passes: `npm run check`

**Tests**: none
**Gate**: `npm run check`

---

### T23: Background health polling (GUI)

**What**: Implement periodic health re-evaluation as a Tauri background task, configurable interval.
**Where**: `src-tauri/src/lib.rs` (or dedicated background module)
**Depends on**: T22
**Reuses**: Scheduler tokio pattern from T15
**Requirement**: POLL-01, POLL-02, POLL-03, GUI-HEALTH-05

**Tools**:
- Skill: `tauri`

**Done when**:
- [ ] Tokio task spawned on app start that evaluates health every N minutes (configurable)
- [ ] Task reads polling interval from Config (default 5min)
- [ ] Task stops when app exits (CancellationToken or AbortHandle)
- [ ] Results written to health cache
- [ ] Frontend can trigger immediate refresh (already in T13)
- [ ] Gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Tests**: unit (task spawn/cancel logic)
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

### T24: Integration testing + edge cases

**What**: End-to-end integration tests covering cross-module flows: scheduler → execution → health → notification chain.
**Where**: `crates/gitty-core/tests/integration_m5.rs`
**Depends on**: T23
**Reuses**: Existing test patterns (tempdir repos)
**Requirement**: All (integration verification)

**Tools**:
- Skill: `tdd`

**Done when**:
- [ ] Test: scheduler `should_run` → execute macro → health evaluated → notification generated
- [ ] Test: corrupt health.json → graceful recovery
- [ ] Test: stale PID file → scheduler starts successfully
- [ ] Test: battery below threshold → scheduler pauses
- [ ] Test: empty repo → health checks skip appropriately
- [ ] Full workspace gate passes: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: ≥5 new integration tests

**Tests**: integration
**Gate**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 → T2 → T3

Phase 2 (Parallel after T3):
  ├── T4 [P] (health cache)
  ├── T5 [P] (changes module)
  ├── T6 [P] (scheduler core)
  T6 → T7 (scheduler daemon, sequential after T6)
  T3 → T8 (notification module, parallel with T4/T5/T6)
  All → T9 (wiring, after all Phase 2 tasks)

Phase 3 (Sequential):
  T9 → T10 → T11 → T12

Phase 4 (Parallel after T12):
  ├── T13 [P] (health IPC)
  ├── T14 [P] (changes IPC)
  ├── T15 [P] (scheduler IPC)
  └── T16 [P] (notification IPC)
  All → T17 (TS types, after all Phase 4)

Phase 5 (Parallel after T17):
  ├── T18 [P] (health page)
  ├── T19 [P] (changes page)
  ├── T20 [P] (notification panel)
  └── T21 [P] (settings extension)
  All → T22 (sidebar nav, after all Phase 5)

Phase 6 (Sequential):
  T22 → T23 → T24
```

---

## Task Granularity Check

| Task | Scope | Status |
| --- | --- | --- |
| T1: Health trait + models | 1 module (types only) | ✅ Granular |
| T2: Config extensions | 2 files (add fields) | ✅ Granular |
| T3: 4 Health Check impls + evaluate fns | 1 module (related impls) | ⚠️ Cohesive — all checks are small, same file |
| T4: Health cache | 1 module | ✅ Granular |
| T5: Changes module | 1 module | ✅ Granular |
| T6: Scheduler core | 1 module (pure logic) | ✅ Granular |
| T7: Scheduler daemon | 1 submodule | ✅ Granular |
| T8: Notification module | 1 module | ✅ Granular |
| T9: Wiring (lib.rs + Cargo.toml) | 2 files (pub mod + deps) | ✅ Granular |
| T10: CLI health command | 1 subcommand | ✅ Granular |
| T11: CLI scheduler command | 1 subcommand | ✅ Granular |
| T12: Public API exports | 1 file | ✅ Granular |
| T13: Health IPC | 1 file (3 commands) | ✅ Granular |
| T14: Changes IPC | 1 file (1 command + cache) | ✅ Granular |
| T15: Scheduler IPC | 1 file (2 commands + bg task) | ✅ Granular |
| T16: Notification IPC | 1 file (4 commands) | ✅ Granular |
| T17: TS types | 3 files (types only) | ✅ Granular |
| T18: Health page | 1 page | ✅ Granular |
| T19: Changes page | 1 page | ✅ Granular |
| T20: Notification panel | 1 component | ✅ Granular |
| T21: Settings extensions | 1 page (add sections) | ✅ Granular |
| T22: Sidebar nav | 1 component (add links) | ✅ Granular |
| T23: Background polling | 1 task setup | ✅ Granular |
| T24: Integration tests | 1 test file | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (body) | Diagram Shows | Status |
| --- | --- | --- | --- |
| T1 | None | Start node | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T1 | T2 → T3 (sequential in Phase 1) | ✅ Match |
| T4 | T3 | T3 → T4 [P] | ✅ Match |
| T5 | T3 | T3 → T5 [P] | ✅ Match |
| T6 | T2 | T3 → T6 [P] (T2 done by Phase 1) | ✅ Match |
| T7 | T6 | T6 → T7 | ✅ Match |
| T8 | T3 | T3 → T8 | ✅ Match |
| T9 | T4,T5,T6,T7,T8 | All Phase 2 → T9 | ✅ Match |
| T10 | T9 | T9 → T10 | ✅ Match |
| T11 | T10 | T10 → T11 | ✅ Match |
| T12 | T10,T11 | T11 → T12 | ✅ Match |
| T13 | T12 | T12 → T13 [P] | ✅ Match |
| T14 | T12 | T12 → T14 [P] | ✅ Match |
| T15 | T12 | T12 → T15 [P] | ✅ Match |
| T16 | T12 | T12 → T16 [P] | ✅ Match |
| T17 | T13,T14,T15,T16 | All Phase 4 → T17 | ✅ Match |
| T18 | T17 | T17 → T18 [P] | ✅ Match |
| T19 | T17 | T17 → T19 [P] | ✅ Match |
| T20 | T17 | T17 → T20 [P] | ✅ Match |
| T21 | T17 | T17 → T21 [P] | ✅ Match |
| T22 | T18,T19,T20,T21 | All Phase 5 → T22 | ✅ Match |
| T23 | T22 | T22 → T23 | ✅ Match |
| T24 | T23 | T23 → T24 | ✅ Match |

---

## Test Co-location Validation

| Task | Code Layer | Requires | Task Says | Status |
| --- | --- | --- | --- | --- |
| T1 | Core domain types | unit | unit | ✅ OK |
| T2 | Config (existing layer) | unit | unit (existing tests) | ✅ OK |
| T3 | Core logic (health checks) | unit | unit | ✅ OK |
| T4 | I/O module (cache) | unit+integration | unit+integration | ✅ OK |
| T5 | I/O module (git2 revwalk) | integration | integration | ✅ OK |
| T6 | Core logic (scheduler) | unit | unit | ✅ OK |
| T7 | I/O module (daemon/process) | unit+integration | unit+integration | ✅ OK |
| T8 | Core logic (notifications) | unit | unit | ✅ OK |
| T9 | Wiring (no logic) | none | none | ✅ OK |
| T10 | CLI command | unit | unit | ✅ OK |
| T11 | CLI command | unit | unit | ✅ OK |
| T12 | Wiring (no logic) | none | none | ✅ OK |
| T13 | Tauri IPC | unit | unit | ✅ OK |
| T14 | Tauri IPC | unit | unit | ✅ OK |
| T15 | Tauri IPC | unit | unit | ✅ OK |
| T16 | Tauri IPC | unit | unit | ✅ OK |
| T17 | TS types (no logic) | none | none | ✅ OK |
| T18 | Svelte page | none (visual) | none | ✅ OK |
| T19 | Svelte page | none (visual) | none | ✅ OK |
| T20 | Svelte component | none (visual) | none | ✅ OK |
| T21 | Svelte page | none (visual) | none | ✅ OK |
| T22 | Svelte component (trivial) | none | none | ✅ OK |
| T23 | Tauri background task | unit | unit | ✅ OK |
| T24 | Integration tests | integration | integration | ✅ OK |
