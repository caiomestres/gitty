# M6 GUI Completion — Tasks

## Task Overview

| # | Task | Priority | Depends | Parallel |
|---|------|----------|---------|----------|
| T1 | Settings: Complete scheduler config | P1 | — | [P] |
| T2 | Notification panel: Actions + navigation | P1 | — | [P] |
| T3 | Health dashboard: Drill-down navigation | P1 | — | [P] |
| T4 | Repo detail: Health summary section | P1 | — | [P] |
| T5 | Repo detail: Fix Ungrouped assignment | P2 | — | [P] |
| T6 | Repo detail: Changed files list | P2 | — | [P] |
| T7 | Job results: Per-step output | P2 | — | [P] |
| T8 | Groups page: Tree visualization | P2 | — | [P] |
| T9 | Sidebar: Empty state | P3 | — | [P] |
| T10 | Settings: Advanced scheduler mode | P3 | T1 | — |

All tasks are frontend-only (Svelte/TypeScript). No backend changes needed — all required IPC commands and data fields exist.

---

## T1: Settings — Complete Scheduler Config

**What**: The scheduler section in Settings currently hardcodes `interval: 30`, `power: { pause_on_battery: true, battery_threshold: 20 }`, and `macro_id: null`. Load all fields from `get_scheduler_status` and save via `set_scheduler_config`.

**Where**: `src/routes/settings/+page.svelte`

**Done when**:
- Scheduler interval loads from backend value, not hardcoded 30
- Pause-on-battery checkbox, battery threshold input, and macro dropdown are displayed
- Macro dropdown loads from `list_macros`; shows "Default (fetch all)" when empty or no selection
- All fields save on change via `set_scheduler_config`
- Reloads on `config-changed` event

**Reqs**: GUI-SCHED-01 through GUI-SCHED-06

**Tests**: Manual — open Settings, change interval, toggle battery, select macro, restart app, verify values persisted.

**Gate**: `npm run check`

**Status**: Done

---

## T2: Notification Panel — Actions + Navigation

**What**: Add "Mark all read" button, visual dimming for read notifications, and click-to-navigate (health notifications → `/health`, scheduler notifications → `/health`).

**Where**: `src/lib/components/NotificationPanel.svelte`

**Done when**:
- "Mark all read" button visible when unread notifications exist
- Clicking it marks all as read via existing `mark_notification_read` command (called per notification)
- Read notifications have muted styling (lower opacity or color)
- Clicking a notification navigates to `/health` (all current notification types relate to health)
- Badge count updates immediately after mark-all-read

**Reqs**: GUI-NOTIF-01 through GUI-NOTIF-05

**Tests**: Manual — trigger notifications via scheduler; click "Mark all read"; verify badge clears; click notification; verify navigation.

**Gate**: `npm run check`

**Status**: Done

---

## T3: Health Dashboard — Drill-Down Navigation

**What**: Make Repository rows in the Health dashboard clickable, navigating to `/repo/[id]`.

**Where**: `src/routes/health/+page.svelte`

**Done when**:
- Repository names in the health table are rendered as `<a>` links to `/repo/[id]`
- Link styling matches the dashboard repo table (hover color change)
- Clicking navigates without full page reload

**Reqs**: GUI-DRILL-01, GUI-DRILL-02

**Tests**: Manual — open Health dashboard; click a repo row; verify navigation to repo detail.

**Gate**: `npm run check`

**Status**: Done

---

## T4: Repository Detail — Health Summary Section

**What**: Add a health checks section to the Repository detail page showing per-check results with severity indicators.

**Where**: `src/routes/repo/[id]/+page.svelte`

**IPC**: `get_repository_health` (already registered, takes `repoId`)

**Done when**:
- Active repos show a "Health" section below the status cards
- Section displays each check name + severity (healthy/warning/critical) with color-coded dots
- If health data is not available, shows "No health data" with a "Refresh" button
- Refresh button calls `refresh_health` then reloads health data
- Missing repos skip the health section

**Reqs**: GUI-REPO-HEALTH-01 through GUI-REPO-HEALTH-03

**Tests**: Manual — open detail for a stale repo; verify stale check shows warning; click refresh; verify update.

**Gate**: `npm run check`

**Status**: Done

---

## T5: Repository Detail — Fix Ungrouped Assignment

**What**: The Group dropdown's `handleGroupChange` returns early when `newGroupId` is empty. Add "Ungrouped" as a selectable option that assigns to the default Ungrouped group.

**Where**: `src/routes/repo/[id]/+page.svelte`

**Done when**:
- Group dropdown includes "Ungrouped" as the first option
- Selecting "Ungrouped" calls `assign_repo_to_group` with the Ungrouped group's actual UUID
- The Ungrouped group's UUID is obtained from `list_groups` (the one with `name === "Ungrouped"`)

**Reqs**: GUI-REPO-UNGROUP-01, GUI-REPO-UNGROUP-02

**Tests**: Manual — assign repo to Group A; switch dropdown to Ungrouped; verify assignment.

**Gate**: `npm run check`

**Status**: Done

---

## T6: Repository Detail — Changed Files List

**What**: Show the list of changed file paths when a Repository is dirty (currently only the count is shown). The file paths are already available in the core `RepositoryStatus.changed_files` field but not exposed in `RepoStatusDto`.

**Where**:
- Backend: `src-tauri/src/commands/workspace.rs` — add `changed_files` field to `RepoStatusDto`
- Frontend: `src/routes/repo/[id]/+page.svelte` — render the file list

**Done when**:
- `RepoStatusDto` includes `changed_files: Vec<ChangedFileDto>` (path + status string)
- Repo detail page shows a collapsible "Changed files" section when dirty
- Files listed with their status (modified, added, deleted, etc.)
- List truncated at 20 files with "show N more" toggle

**Reqs**: GUI-REPO-FILES-01, GUI-REPO-FILES-02

**Tests**: Manual — create dirty repo with 5 changed files; open detail; verify paths displayed.

**Gate**: `npm run check && cargo test -p gitty-tauri && cargo clippy -- -D warnings`

**Status**: Done

---

## T7: Job Results — Per-Step Output

**What**: The `JobResults` component shows repo-level summary but not per-step detail. The `StepResultDto` already has `output` and `status` fields — just not rendered.

**Where**: `src/lib/components/JobResults.svelte`

**Done when**:
- Each repo row is expandable to show step results
- Each step shows: index, status (with icon), and output (collapsed by default)
- Failed steps are highlighted with error styling
- Single-step Jobs show step output inline without expand/collapse

**Reqs**: GUI-JOB-01 through GUI-JOB-03

**Tests**: Manual — run a 2-step macro where step 2 fails; verify step-level output visible with error highlighting.

**Gate**: `npm run check`

**Status**: Done

---

## T8: Groups Page — Tree Visualization

**What**: Replace the flat table with an indented tree that reflects the Group hierarchy. Use the existing `group_tree` IPC command which returns nested `GroupTreeNodeDto`.

**Where**: `src/routes/groups/+page.svelte`

**Done when**:
- Groups displayed as indented tree (using padding-left per depth level)
- Parent Groups show collapse/expand toggle
- Childless, repo-less Groups show muted "(empty)" label
- CRUD actions (rename, delete, move) still work on each tree node
- Create dialog still allows selecting a parent Group

**Reqs**: GUI-TREE-01 through GUI-TREE-03

**Tests**: Manual — create nested Groups A > B > C; open Groups page; verify indented tree; collapse A; verify B/C hidden.

**Gate**: `npm run check`

**Status**: Done

---

## T9: Sidebar — Empty State

**What**: When no Repositories exist, the sidebar Explorer section is hidden. Show a helpful empty state instead.

**Where**: `src/lib/components/Sidebar.svelte`

**Done when**:
- When the group tree has no repos, sidebar shows "No repositories. Scan a directory to get started." with a link to `/settings`
- When Groups exist but have no repos, each Group node shows "(empty)"

**Reqs**: GUI-SIDEBAR-01, GUI-SIDEBAR-02

**Tests**: Manual — fresh config with no repos; open app; verify sidebar shows empty state with link.

**Gate**: `npm run check`

**Status**: Done

---

## T10: Settings — Advanced Scheduler Mode

**What**: Add UI for advanced scheduler triggers (time window start/end, day-of-week checkboxes).

**Where**: `src/routes/settings/+page.svelte`

**Depends on**: T1 (basic scheduler config must be complete first)

**Done when**:
- Toggle between "Simple" and "Advanced" scheduler mode
- Advanced shows: time window start (HH:MM), time window end (HH:MM), day-of-week checkboxes
- Midnight-crossing windows show a note: "Window wraps past midnight"
- All fields save on change via `set_scheduler_config`

**Reqs**: GUI-SCHED-ADV-01 through GUI-SCHED-ADV-03

**Tests**: Manual — set advanced mode with 22:00–06:00 Mon–Fri; restart app; verify values persisted.

**Gate**: `npm run check`

**Status**: Done
