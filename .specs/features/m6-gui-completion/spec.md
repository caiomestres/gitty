# M6 Sub-feature: GUI Completion — Specification

## Problem Statement

The M4 and M5 GUI pages exist and are substantially wired to backend commands, but several have partial implementations: hardcoded defaults where backend data should be loaded, missing user actions (mark all read, dismiss, retry), no navigation between related views (health → repo, notification → source), and cosmetic gaps (flat group table instead of tree, no empty states). These gaps prevent the GUI from being a polished, ship-ready product.

## Goals

- [x] Every GUI control reads its initial value from the backend (no hardcoded defaults)
- [x] Every list view with actionable items provides the expected actions (mark read, dismiss, retry, navigate)
- [x] Cross-page navigation works (health drill-down → repo detail, notification → source page)
- [x] Empty states are handled gracefully across all pages and components
- [x] Settings page exposes all scheduler and notification options that the backend supports

## Out of Scope

| Feature | Reason |
| --- | --- |
| Native file picker for Scan Root | Deferred idea — `tauri-plugin-dialog` |
| Drag-and-drop reorder in Groups page | Nice-to-have; move dialog already works |
| Condition expression autocomplete in Macro editor | Would require a condition parser; free-text is sufficient for v1 |
| Push notifications to mobile | Desktop-only per PROJECT.md |
| Real-time event subscription (WebSocket/SSE) | Polling is sufficient for v1 |
| Variable overrides at Macro runtime | Variables are set at definition time; runtime override deferred |

---

## User Stories

### P1: Settings Scheduler — Complete Configuration ⭐ MVP

**User Story**: As a developer, I want the Settings scheduler section to load and save all scheduler fields (interval, power settings, macro selection) so that I can fully configure automation from the GUI.

**Acceptance Criteria**:

1. WHEN Settings loads THEN system SHALL read the current scheduler config from the backend and populate all fields (enabled, interval, pause_on_battery, battery_threshold, macro_id)
2. WHEN the user changes the scheduler interval THEN system SHALL persist the updated value to Config
3. WHEN the user toggles "pause on battery" THEN system SHALL persist the updated value to Config
4. WHEN the user changes the battery threshold THEN system SHALL persist the updated value to Config
5. WHEN the user selects a Macro from the dropdown THEN system SHALL persist the selected macro_id to Config
6. WHEN no Macros are defined THEN system SHALL show "Default (fetch all)" as the only option
7. WHEN the scheduler config changes externally (file watcher) THEN system SHALL reload the displayed values

**Independent Test**: Open Settings, change interval from 30 to 60, change macro to a custom one; restart app; verify values persisted.

---

### P1: Notification Panel — Actions ⭐ MVP

**User Story**: As a developer, I want to mark all notifications as read, dismiss individual notifications, and navigate to the source of a notification so that the panel is actionable, not just informational.

**Acceptance Criteria**:

1. WHEN the panel has unread notifications THEN system SHALL show a "Mark all read" button
2. WHEN "Mark all read" is clicked THEN system SHALL mark every notification as read and update the badge count
3. WHEN a notification about health is clicked THEN system SHALL navigate to the Health dashboard
4. WHEN a notification about a scheduler run is clicked THEN system SHALL navigate to the Health dashboard
5. WHEN a notification is read THEN system SHALL visually dim it (lower opacity or muted text)

**Independent Test**: Trigger 3 notifications; click "Mark all read"; verify badge clears and all items are dimmed.

---

### P1: Health Dashboard — Drill-Down Navigation ⭐ MVP

**User Story**: As a developer, I want to click a Repository in the Health dashboard to navigate to its detail page so that I can investigate and act on health issues.

**Acceptance Criteria**:

1. WHEN a Repository row in the Health table is clicked THEN system SHALL navigate to `/repo/[id]`
2. WHEN viewing the Health dashboard THEN system SHALL show Repository names as clickable links

**Independent Test**: Open Health dashboard; click a repo row; verify navigation to repo detail.

---

### P1: Repository Detail — Health Summary ⭐ MVP

**User Story**: As a developer, I want to see health check results on the Repository detail page so that I have full context without switching to the Health dashboard.

**Acceptance Criteria**:

1. WHEN the Repository detail page loads for an active Repository THEN system SHALL display health check results (stale, diverged, dirty, detached) with severity indicators
2. WHEN health data is not cached THEN system SHALL show a "No health data — run a health check" message with a refresh button
3. WHEN the refresh button is clicked THEN system SHALL trigger a fresh health evaluation for that Repository

**Independent Test**: Open repo detail for a stale repo; verify stale check shows warning severity.

---

### P2: Repository Detail — Fix Ungrouped Assignment

**User Story**: As a developer, I want to move a Repository back to Ungrouped so that I can unassign it from a Group.

**Acceptance Criteria**:

1. WHEN the Group dropdown is shown THEN system SHALL include "Ungrouped" as a selectable option
2. WHEN "Ungrouped" is selected THEN system SHALL assign the Repository to the default Ungrouped group

**Independent Test**: Assign repo to Group A; change dropdown to Ungrouped; verify repo is now in Ungrouped.

---

### P2: Repository Detail — Changed Files List

**User Story**: As a developer, I want to see the list of changed files (not just the count) on the Repository detail page so that I know what's dirty.

**Acceptance Criteria**:

1. WHEN a Repository has uncommitted changes THEN system SHALL display a collapsible list of changed file paths
2. WHEN the list is long (>20 files) THEN system SHALL truncate with a "show N more" toggle

**Independent Test**: Create dirty repo with 5 changed files; open detail; verify file paths displayed.

---

### P2: Job Results — Per-Step Output

**User Story**: As a developer, I want to see per-step output in the Job results so that I can diagnose failures without re-running commands manually.

**Acceptance Criteria**:

1. WHEN a Job has step results THEN system SHALL display each step's status and output (expandable)
2. WHEN a step failed THEN system SHALL highlight it with error styling and show the error category
3. WHEN a Job has a single step THEN system SHALL show step output inline (no expand/collapse)

**Independent Test**: Run a macro with 2 steps where step 2 fails; verify step-level output is visible.

---

### P2: Groups Page — Tree Visualization

**User Story**: As a developer, I want the Groups admin page to show Groups as an indented tree so that the hierarchy is visually clear.

**Acceptance Criteria**:

1. WHEN Groups are nested THEN system SHALL display them with indentation reflecting depth
2. WHEN a Group has children THEN system SHALL show a collapse/expand toggle
3. WHEN a Group has no Repositories and no children THEN system SHALL show it with a muted "empty" indicator

**Independent Test**: Create Groups A > B > C; open Groups page; verify indented tree display.

---

### P3: Sidebar — Empty State

**User Story**: As a developer, I want the sidebar Explorer section to show a helpful message when no Repositories exist so that I know to scan a directory.

**Acceptance Criteria**:

1. WHEN the group tree is empty THEN system SHALL display "No repositories. Scan a directory to get started." with a link to Settings
2. WHEN the group tree has Groups but no Repositories THEN system SHALL display the Groups with "(empty)" labels

**Independent Test**: Fresh install; open app; verify sidebar shows empty state message.

---

### P3: Settings — Scheduler Advanced Mode

**User Story**: As a developer, I want to configure advanced scheduler triggers (time window, days of week) from the GUI so that I can use the full scheduling capability without the CLI.

**Acceptance Criteria**:

1. WHEN the user selects "Advanced" mode THEN system SHALL show time window inputs (start time, end time) and day-of-week checkboxes
2. WHEN advanced trigger fields change THEN system SHALL persist to Config on change
3. WHEN the time window crosses midnight THEN system SHALL display a note explaining the wrap-around behavior

**Independent Test**: Set advanced mode with 22:00–06:00 window, Mon–Fri; verify saved and reloaded correctly.

---

## Edge Cases

- WHEN a notification references a Repository that has since been removed THEN system SHALL navigate to the Health dashboard (fallback)
- WHEN the scheduler macro_id references a Macro that was deleted THEN system SHALL show "Default (fetch all)" and log a warning
- WHEN health data fails to load on the Repository detail page THEN system SHALL show the error inline, not crash the page
- WHEN the group tree has deeply nested Groups (>5 levels) THEN system SHALL render correctly with scroll

---

## Requirement Traceability

| Requirement ID | Story | Priority | Status |
| --- | --- | --- | --- |
| GUI-SCHED-01 | Settings Scheduler — load all fields | P1 | Done |
| GUI-SCHED-02 | Settings Scheduler — save interval | P1 | Done |
| GUI-SCHED-03 | Settings Scheduler — save power settings | P1 | Done |
| GUI-SCHED-04 | Settings Scheduler — save macro selection | P1 | Done |
| GUI-SCHED-05 | Settings Scheduler — default macro label | P1 | Done |
| GUI-SCHED-06 | Settings Scheduler — reload on external change | P1 | Done |
| GUI-NOTIF-01 | Notification Panel — mark all read button | P1 | Done |
| GUI-NOTIF-02 | Notification Panel — mark all read action | P1 | Done |
| GUI-NOTIF-03 | Notification Panel — navigate to health | P1 | Done |
| GUI-NOTIF-04 | Notification Panel — navigate on scheduler notif | P1 | Done |
| GUI-NOTIF-05 | Notification Panel — read visual dim | P1 | Done |
| GUI-DRILL-01 | Health Dashboard — click navigates to repo | P1 | Done |
| GUI-DRILL-02 | Health Dashboard — repo names clickable | P1 | Done |
| GUI-REPO-HEALTH-01 | Repo Detail — show health checks | P1 | Done |
| GUI-REPO-HEALTH-02 | Repo Detail — no data message | P1 | Done |
| GUI-REPO-HEALTH-03 | Repo Detail — refresh button | P1 | Done |
| GUI-REPO-UNGROUP-01 | Repo Detail — Ungrouped in dropdown | P2 | Done |
| GUI-REPO-UNGROUP-02 | Repo Detail — assign to Ungrouped | P2 | Done |
| GUI-REPO-FILES-01 | Repo Detail — changed files list | P2 | Done |
| GUI-REPO-FILES-02 | Repo Detail — truncate long list | P2 | Done |
| GUI-JOB-01 | Job Results — per-step output | P2 | Done |
| GUI-JOB-02 | Job Results — failed step styling | P2 | Done |
| GUI-JOB-03 | Job Results — single step inline | P2 | Done |
| GUI-TREE-01 | Groups Page — indented tree | P2 | Done |
| GUI-TREE-02 | Groups Page — collapse/expand | P2 | Done |
| GUI-TREE-03 | Groups Page — empty indicator | P2 | Done |
| GUI-SIDEBAR-01 | Sidebar — empty state message | P3 | Done |
| GUI-SIDEBAR-02 | Sidebar — empty Groups label | P3 | Done |
| GUI-SCHED-ADV-01 | Settings — advanced mode UI | P3 | Done |
| GUI-SCHED-ADV-02 | Settings — save advanced trigger | P3 | Done |
| GUI-SCHED-ADV-03 | Settings — midnight crossing note | P3 | Done |

**Coverage:** 31 requirements, 31 verified

---

## Success Criteria

- [x] Settings scheduler section loads all fields from backend and round-trips correctly
- [x] Notification panel supports mark-all-read and navigates to source pages
- [x] Health dashboard rows are clickable and navigate to repo detail
- [x] Repository detail page displays health check results inline
- [x] All pages handle empty/loading/error states gracefully
