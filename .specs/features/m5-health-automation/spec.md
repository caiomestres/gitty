# M5: Health, Dashboard & Automation — Specification

## Problem Statement

Developers managing large Repository collections have no visibility into workspace staleness, no automated refresh mechanism, and no historical activity view. They cannot tell at a glance which Repositories are drifting, who changed what recently, or rely on automated background synchronization. Manual vigilance doesn't scale beyond a handful of Repositories.

## Goals

- [ ] Per-Repository health evaluation with actionable severity (healthy/warning/critical)
- [ ] Single-number Workspace Health score for at-a-glance freshness judgment
- [ ] Time-windowed activity view across all Repositories (commits, authors, branches)
- [ ] Background automation that keeps Repositories fresh without manual intervention
- [ ] Proactive notification of critical health degradation

## Out of Scope

| Feature | Reason |
| --- | --- |
| User-defined Health Check plugins | Trait prepared; only 4 built-in checks ship in v1 |
| Team grouping in Change Dashboard | Requires Author-to-Team mapping; deferred to v2 |
| Dependency Map integration with health | Deferred per CONTEXT.md (v2) |
| Push notifications to mobile/external | Desktop-only in v1 |
| Dynamic resource-aware concurrency | Fixed concurrency model from M1 |
| Multiple scheduler profiles | Single schedule configuration per Workspace |

---

## User Stories

### P1: Health Check Evaluation ⭐ MVP

**User Story**: As a developer, I want each Repository to be evaluated against health criteria (staleness, divergence, dirty tree, detached HEAD) so that I can see which Repositories need attention.

**Why P1**: Core value proposition — visibility into workspace freshness.

**Acceptance Criteria**:

1. WHEN a health evaluation runs THEN system SHALL evaluate each active Repository against all enabled Health Checks
2. WHEN the HEAD commit is older than the configured stale threshold (default 7 days) THEN system SHALL report that Repository's Stale check as warning (>threshold) or critical (>2x threshold)
3. WHEN a Repository is behind its upstream by more than the configured threshold THEN system SHALL report Diverged as warning (>5 behind) or critical (>20 behind)
4. WHEN a Repository has uncommitted changes (dirty tree) THEN system SHALL report Dirty as warning
5. WHEN a Repository is in detached HEAD state THEN system SHALL report Detached as critical
6. WHEN a Repository is in Missing state THEN system SHALL exclude it from health evaluation
7. WHEN a Repository has no upstream configured THEN system SHALL skip the Diverged check (not penalize)

**Independent Test**: Run `gitty health` against a workspace with repos in various states; verify each check reports correct severity.

---

### P1: Workspace Health Score ⭐ MVP

**User Story**: As a developer, I want a single aggregate Workspace Health score so that I can judge overall workspace freshness at a glance.

**Why P1**: Without aggregation, checking N repos individually doesn't scale.

**Acceptance Criteria**:

1. WHEN health evaluation completes THEN system SHALL compute score as `(repos_not_critical / total_active_repos) * 100`
2. WHEN all Repositories are healthy THEN system SHALL display score as 100%
3. WHEN the score is computed THEN system SHALL exclude Missing Repositories from both numerator and denominator
4. WHEN there are zero active Repositories THEN system SHALL display score as N/A (not 0%)

**Independent Test**: Create workspace with 10 repos, make 3 critical; verify score shows 70%.

---

### P1: Health CLI ⭐ MVP

**User Story**: As a developer, I want to check workspace health from the terminal so that I can integrate health monitoring into my workflow without opening the GUI.

**Why P1**: CLI parity is a project principle (D55).

**Acceptance Criteria**:

1. WHEN user runs `gitty health` THEN system SHALL display the Workspace Health score and per-Repository check results
2. WHEN user runs `gitty health --repo <id-or-name>` THEN system SHALL display detailed checks for that single Repository
3. WHEN health data is cached (from a prior GUI/scheduler run) THEN system SHALL display cached data with a "last evaluated: <timestamp>" label
4. WHEN no cached health data exists THEN system SHALL perform a fresh evaluation

**Independent Test**: Run `gitty health` and verify tabular output with score header.

---

### P1: Health Cache Persistence ⭐ MVP

**User Story**: As a developer, I want health results cached so that the CLI can show last-known health without re-scanning every time, and the GUI can show results immediately on launch.

**Why P1**: Performance — health evaluation touches every repo via git2; caching avoids repeated work.

**Acceptance Criteria**:

1. WHEN health evaluation completes THEN system SHALL persist results to `health.json` in the Config directory
2. WHEN writing health.json THEN system SHALL use atomic temp-file + rename (same pattern as Config)
3. WHEN multiple processes attempt concurrent writes THEN system SHALL use an advisory file lock to prevent corruption
4. WHEN health.json is read THEN system SHALL include a `last_evaluated` timestamp (RFC3339) in the displayed data
5. WHEN health.json does not exist THEN system SHALL treat it as "never evaluated" (trigger fresh evaluation)

**Independent Test**: Evaluate health, read health.json, verify structure and timestamp.

---

### P1: Scheduler Engine ⭐ MVP

**User Story**: As a developer, I want a background scheduler that automatically runs Macros on my workspace so that my Repositories stay fresh without manual intervention.

**Why P1**: The primary automation value — freshness without effort.

**Acceptance Criteria**:

1. WHEN the scheduler is enabled and the current time matches trigger conditions THEN system SHALL execute the configured Macro
2. WHEN no Macro is configured THEN system SHALL run the system default `__scheduler_default` (fetch on all repos)
3. WHEN the scheduler completes a Macro run THEN system SHALL trigger a health re-evaluation
4. WHEN the scheduler is hosted in the GUI THEN system SHALL run as a tokio background task
5. WHEN the scheduler is hosted in the CLI THEN system SHALL self-daemonize (detach from terminal, write PID file)
6. WHEN a scheduler instance is already running (PID file exists and process alive) THEN system SHALL refuse to start a second instance
7. WHEN `gitty scheduler stop` is invoked THEN system SHALL signal the daemon to terminate gracefully
8. WHEN `gitty scheduler status` is invoked THEN system SHALL report whether the scheduler is running, last run time, and next scheduled run

**Independent Test**: Start scheduler with 5-second interval, verify fetch macro executes, verify PID file created.

---

### P1: Scheduler Triggers ⭐ MVP

**User Story**: As a developer, I want to configure when the scheduler runs (interval, time window, days) so that automation respects my work patterns.

**Why P1**: Without configurable triggers, the scheduler is either too aggressive or useless.

**Acceptance Criteria**:

1. WHEN simple mode is configured THEN system SHALL run the Macro every N minutes/hours (configurable interval)
2. WHEN advanced mode is configured THEN system SHALL only run within the specified time window (HH:MM to HH:MM) on specified days of the week
3. WHEN outside the time window THEN system SHALL skip the scheduled run and wait for the next interval within the window
4. WHEN the system is on battery and below the configured threshold THEN system SHALL pause scheduled runs
5. WHEN the system returns to AC power or above threshold THEN system SHALL resume scheduled runs
6. WHEN scheduler configuration changes THEN system SHALL apply the new schedule immediately (no restart required)

**Independent Test**: Configure advanced mode with time window; verify runs only occur within window.

---

### P1: Change Dashboard Data ⭐ MVP

**User Story**: As a developer, I want to see what changed across my workspace over configurable time windows so that I can track activity and understand what happened while I was away.

**Why P1**: Core visibility feature — complements health with activity context.

**Acceptance Criteria**:

1. WHEN the Change Dashboard is opened THEN system SHALL scan recent commits from all active Repositories using git2 Revwalk
2. WHEN scanning THEN system SHALL collect: commit hash, author, date, subject, branch, repository
3. WHEN a time window is selected (24h, 7d, 30d) THEN system SHALL filter commits by author-date within that window
4. WHEN the default view loads THEN system SHALL show HEAD branch only for each Repository
5. WHEN "show all branches" is toggled for a Repository THEN system SHALL include commits from all local branches with upstream tracking
6. WHEN grouping by Author is selected THEN system SHALL aggregate commits by author name
7. WHEN grouping by Repository is selected THEN system SHALL aggregate commits by Repository
8. WHEN grouping by Branch is selected THEN system SHALL aggregate commits by branch name
9. WHEN change data is in memory cache THEN system SHALL serve from cache without re-scanning
10. WHEN a fetch or pull completes THEN system SHALL invalidate the cached change data

**Independent Test**: Create repos with known commit history; open dashboard with 7d window; verify correct commits shown grouped by author.

---

### P2: Health Dashboard (GUI)

**User Story**: As a developer, I want a visual Workspace Health dashboard in the GUI so that I can see health status and drill down into problematic Repositories.

**Why P2**: GUI representation of P1 health data; P1 (engine + CLI) is independently valuable.

**Acceptance Criteria**:

1. WHEN the Health dashboard loads THEN system SHALL display the aggregate Workspace Health score prominently
2. WHEN viewing the dashboard THEN system SHALL show per-Repository health status (traffic light: green/yellow/red)
3. WHEN clicking a Repository row THEN system SHALL navigate to a drill-down showing individual check results
4. WHEN a health refresh button is clicked THEN system SHALL trigger a fresh evaluation and update the display
5. WHEN background polling is enabled (default 5min interval) THEN system SHALL auto-refresh health data periodically

**Independent Test**: Open Health dashboard; verify score and per-repo indicators match CLI output.

---

### P2: Change Dashboard (GUI)

**User Story**: As a developer, I want a visual Change Dashboard in the GUI so that I can browse workspace activity with time-window and grouping controls.

**Why P2**: GUI representation of P1 change data; the data engine is the P1 priority.

**Acceptance Criteria**:

1. WHEN the Change Dashboard page loads THEN system SHALL display commits from the default time window (7d) grouped by Repository
2. WHEN a time window selector is changed THEN system SHALL re-filter and re-display commits
3. WHEN a grouping mode is selected THEN system SHALL reorganize the display accordingly
4. WHEN "show all branches" is toggled THEN system SHALL expand/collapse branch scope for that Repository

**Independent Test**: Open Change Dashboard; switch between time windows and groupings; verify data updates.

---

### P2: Notifications

**User Story**: As a developer, I want to be notified when Repositories enter critical health state so that I can take action without constantly monitoring the dashboard.

**Why P2**: Proactive alerting enhances but doesn't replace the dashboard.

**Acceptance Criteria**:

1. WHEN a health evaluation detects Repositories in critical state AND notifications are enabled THEN system SHALL emit a notification
2. WHEN the notification trigger is "on critical" THEN system SHALL only notify for critical transitions
3. WHEN the notification trigger is "on any change" THEN system SHALL notify for any severity change
4. WHEN the notification trigger is "on scheduler complete" THEN system SHALL notify after every scheduler run
5. WHEN multiple Repositories are critical THEN system SHALL aggregate into a single notification ("3 repos are critical")
6. WHEN the app is in the foreground THEN system SHALL show notifications in the in-app panel
7. WHEN the app is in the background THEN system SHALL emit an OS-native toast (via tauri-plugin-notification)
8. WHEN a notification is older than 7 days THEN system SHALL auto-purge it from the stored list on next Config load

**Independent Test**: Trigger critical state on 2 repos; verify single aggregated toast appears; verify in-app panel shows the notification.

---

### P2: Notification Configuration

**User Story**: As a developer, I want to configure notification behavior so that I'm not overwhelmed by alerts I don't care about.

**Why P2**: Without configuration, notifications are either too noisy or disabled entirely.

**Acceptance Criteria**:

1. WHEN user opens Settings THEN system SHALL show notification trigger preference (critical/any-change/scheduler-complete)
2. WHEN user changes the trigger preference THEN system SHALL persist to Config immediately
3. WHEN `gitty config` is used from CLI THEN system SHALL allow setting notification preferences
4. WHEN notifications are disabled THEN system SHALL not emit any toasts or in-app notifications

**Independent Test**: Change trigger from "critical" to "disabled" in settings; verify no notifications fire.

---

### P3: Health Background Polling (GUI)

**User Story**: As a developer, I want the GUI to periodically re-evaluate health in the background so that the dashboard stays current without manual refresh.

**Why P3**: Convenience enhancement; on-demand + scheduler already provide coverage.

**Acceptance Criteria**:

1. WHEN the GUI is running and polling is enabled THEN system SHALL re-evaluate health every N minutes (configurable, default 5)
2. WHEN polling interval is changed in Settings THEN system SHALL apply the new interval without restart
3. WHEN the GUI exits THEN system SHALL stop polling (no orphan tasks)

**Independent Test**: Set polling to 10 seconds in dev; verify health.json updates automatically.

---

## Edge Cases

- WHEN a Repository's `.git` directory is corrupt (git2 fails to open) THEN system SHALL report that Repository as critical with error details, not crash
- WHEN health.json is corrupt/malformed THEN system SHALL discard it and treat as "never evaluated"
- WHEN the scheduler PID file exists but the process is dead THEN system SHALL treat it as stale (delete PID file, allow new start)
- WHEN battery detection is unavailable (e.g., desktop without battery) THEN system SHALL treat power condition as "always satisfied" (never pause)
- WHEN a Repository has zero commits (empty repo) THEN system SHALL skip Stale and Diverged checks (only Dirty/Detached applicable)
- WHEN Revwalk encounters a shallow clone THEN system SHALL return whatever commits are available (no error)
- WHEN the time window returns zero commits THEN system SHALL display an empty state (not an error)
- WHEN the scheduler Macro fails on some repos THEN system SHALL still trigger health re-evaluation (partial failure is expected)

---

## Requirement Traceability

| Requirement ID | Story | Phase | Status |
| --- | --- | --- | --- |
| HEALTH-01 | P1: Health Check Evaluation | Design | Pending |
| HEALTH-02 | P1: Health Check Evaluation (Stale) | Design | Pending |
| HEALTH-03 | P1: Health Check Evaluation (Diverged) | Design | Pending |
| HEALTH-04 | P1: Health Check Evaluation (Dirty) | Design | Pending |
| HEALTH-05 | P1: Health Check Evaluation (Detached) | Design | Pending |
| HEALTH-06 | P1: Health Check Evaluation (Missing skip) | Design | Pending |
| HEALTH-07 | P1: Health Check Evaluation (No upstream) | Design | Pending |
| SCORE-01 | P1: Workspace Health Score | Design | Pending |
| SCORE-02 | P1: Workspace Health Score (100%) | Design | Pending |
| SCORE-03 | P1: Workspace Health Score (Missing excluded) | Design | Pending |
| SCORE-04 | P1: Workspace Health Score (zero repos) | Design | Pending |
| CLI-01 | P1: Health CLI (all repos) | Design | Pending |
| CLI-02 | P1: Health CLI (single repo) | Design | Pending |
| CLI-03 | P1: Health CLI (cached) | Design | Pending |
| CLI-04 | P1: Health CLI (fresh if no cache) | Design | Pending |
| CACHE-01 | P1: Health Cache (persist) | Design | Pending |
| CACHE-02 | P1: Health Cache (atomic write) | Design | Pending |
| CACHE-03 | P1: Health Cache (file lock) | Design | Pending |
| CACHE-04 | P1: Health Cache (timestamp) | Design | Pending |
| CACHE-05 | P1: Health Cache (missing = fresh) | Design | Pending |
| SCHED-01 | P1: Scheduler Engine (trigger match) | Design | Pending |
| SCHED-02 | P1: Scheduler Engine (default macro) | Design | Pending |
| SCHED-03 | P1: Scheduler Engine (post-run health) | Design | Pending |
| SCHED-04 | P1: Scheduler Engine (GUI host) | Design | Pending |
| SCHED-05 | P1: Scheduler Engine (CLI daemonize) | Design | Pending |
| SCHED-06 | P1: Scheduler Engine (single instance) | Design | Pending |
| SCHED-07 | P1: Scheduler Engine (stop) | Design | Pending |
| SCHED-08 | P1: Scheduler Engine (status) | Design | Pending |
| TRIG-01 | P1: Scheduler Triggers (simple interval) | Design | Pending |
| TRIG-02 | P1: Scheduler Triggers (advanced window) | Design | Pending |
| TRIG-03 | P1: Scheduler Triggers (outside window skip) | Design | Pending |
| TRIG-04 | P1: Scheduler Triggers (battery pause) | Design | Pending |
| TRIG-05 | P1: Scheduler Triggers (resume on AC) | Design | Pending |
| TRIG-06 | P1: Scheduler Triggers (hot reload) | Design | Pending |
| CHANGE-01 | P1: Change Dashboard Data (scan) | Design | Pending |
| CHANGE-02 | P1: Change Dashboard Data (fields) | Design | Pending |
| CHANGE-03 | P1: Change Dashboard Data (time filter) | Design | Pending |
| CHANGE-04 | P1: Change Dashboard Data (HEAD default) | Design | Pending |
| CHANGE-05 | P1: Change Dashboard Data (all branches toggle) | Design | Pending |
| CHANGE-06 | P1: Change Dashboard Data (group by author) | Design | Pending |
| CHANGE-07 | P1: Change Dashboard Data (group by repo) | Design | Pending |
| CHANGE-08 | P1: Change Dashboard Data (group by branch) | Design | Pending |
| CHANGE-09 | P1: Change Dashboard Data (memory cache) | Design | Pending |
| CHANGE-10 | P1: Change Dashboard Data (invalidate on fetch) | Design | Pending |
| GUI-HEALTH-01 | P2: Health Dashboard (score) | Design | Pending |
| GUI-HEALTH-02 | P2: Health Dashboard (per-repo) | Design | Pending |
| GUI-HEALTH-03 | P2: Health Dashboard (drill-down) | Design | Pending |
| GUI-HEALTH-04 | P2: Health Dashboard (refresh) | Design | Pending |
| GUI-HEALTH-05 | P2: Health Dashboard (auto-poll) | Design | Pending |
| GUI-CHANGE-01 | P2: Change Dashboard GUI (default view) | Design | Pending |
| GUI-CHANGE-02 | P2: Change Dashboard GUI (time selector) | Design | Pending |
| GUI-CHANGE-03 | P2: Change Dashboard GUI (grouping) | Design | Pending |
| GUI-CHANGE-04 | P2: Change Dashboard GUI (branch toggle) | Design | Pending |
| NOTIF-01 | P2: Notifications (emit) | Design | Pending |
| NOTIF-02 | P2: Notifications (critical trigger) | Design | Pending |
| NOTIF-03 | P2: Notifications (any-change trigger) | Design | Pending |
| NOTIF-04 | P2: Notifications (scheduler trigger) | Design | Pending |
| NOTIF-05 | P2: Notifications (aggregate) | Design | Pending |
| NOTIF-06 | P2: Notifications (in-app panel) | Design | Pending |
| NOTIF-07 | P2: Notifications (OS toast) | Design | Pending |
| NOTIF-08 | P2: Notifications (TTL purge) | Design | Pending |
| NOTIF-CFG-01 | P2: Notification Config (settings UI) | Design | Pending |
| NOTIF-CFG-02 | P2: Notification Config (persist) | Design | Pending |
| NOTIF-CFG-03 | P2: Notification Config (CLI) | Design | Pending |
| NOTIF-CFG-04 | P2: Notification Config (disable) | Design | Pending |
| POLL-01 | P3: Health Polling (interval) | Design | Pending |
| POLL-02 | P3: Health Polling (configurable) | Design | Pending |
| POLL-03 | P3: Health Polling (stop on exit) | Design | Pending |

**Coverage:** 60 total requirements, 0 mapped to tasks, 60 unmapped

---

## Success Criteria

- [ ] `gitty health` displays per-repo checks and aggregate score correctly for a 20+ repo workspace
- [ ] Scheduler auto-fetches on configured interval without user intervention for 24+ hours
- [ ] Change Dashboard shows correct commit groupings across 50+ repos within 2 seconds
- [ ] OS notifications fire for critical health transitions without duplicates
- [ ] CLI and GUI show consistent health data from shared health.json cache
