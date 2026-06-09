# M7: UX Revamp — Specification

## Problem Statement

The UI has accumulated UX friction: sidebar and action icons are a mix of colored emoji and Unicode characters with inconsistent sizing. Scan Roots require manual path input (no file picker). Status indicators ("dirty", "missing", "healthy") have no tooltips, forcing users to consult docs. The Dashboard and Changes tables have no pagination, becoming unresponsive with 50+ Repositories. Users cannot remove a Repository from tracking once discovered. The top bar wastes space on a redundant "Fetch All" button and uses an emoji for the notification bell. First-time users see an empty Dashboard with no guidance.

## Goals

- [ ] Consistent monochrome SVG icon set across all UI surfaces
- [ ] Native OS folder picker and drag-and-drop for Scan Root management
- [ ] Tooltips on every status indicator and domain term
- [ ] Configurable pagination for large tables
- [ ] Unregister Repositories from tracking without affecting disk
- [ ] Redesigned top bar with health indicator, global search, and SVG bell
- [ ] First-run onboarding for empty Dashboards

## Out of Scope

| Feature | Reason |
| --- | --- |
| Custom icon upload | Fixed SVG set for v1 |
| Global drag-and-drop | Drop zone on Settings page only (PRD) |
| Keyboard shortcut for search | Not in PRD scope |
| Undo unregister | Rescan recovers the Repository |
| Full search engine (fuzzy, commands) | Simple substring filter in v1 |

---

## User Stories

### P1: SVG Icon Set ⭐ MVP

**User Story**: As a developer, I want all sidebar and action icons to be consistent monochrome SVGs so that the UI looks polished instead of a mix of colored emojis.

**Issues**: #42

**Acceptance Criteria**:

1. WHEN any icon is rendered in the sidebar, action buttons, or status badges THEN it SHALL be a monochrome SVG (not emoji or Unicode character)
2. WHEN an icon is in its default state THEN it SHALL use `--color-muted` fill
3. WHEN an icon is hovered or active THEN it SHALL brighten to `--color-ink`
4. WHEN the icon set is audited THEN all icons SHALL come from a single library (Lucide, Heroicons, or Phosphor — decided during implementation)
5. WHEN the sidebar is rendered THEN navigation items SHALL use SVG icons instead of emoji prefixes
6. WHEN the notification bell is rendered THEN it SHALL be a monochrome SVG (replacing the current `🔔` emoji in NotificationPanel.svelte) (D109)

**Independent Test**: Visual audit of every page; zero emoji or Unicode icons remain (including notification bell).

---

### P1: Native Folder Picker ⭐ MVP

**User Story**: As a developer, I want to add Scan Roots using a native folder picker and drag-and-drop so that I don't have to type filesystem paths manually.

**Issues**: #43

**Acceptance Criteria**:

1. WHEN the user clicks "Add Scan Root" in Settings THEN the system SHALL open the OS native folder picker via `tauri-plugin-dialog` (D104)
2. WHEN the user selects a directory via the picker THEN the system SHALL add it as a Scan Root and trigger a scan
3. WHEN the user drags a folder onto the Scan Roots section in Settings THEN the system SHALL add it as a Scan Root via Tauri's built-in `onDragDropEvent` (D104)
4. WHEN a folder is dragged over the drop zone THEN the system SHALL show a dashed-border visual indicator
5. WHEN the user prefers manual input THEN the system SHALL provide a collapsed "Enter path manually" fallback
6. WHEN a dropped item is not a directory THEN the system SHALL show an error toast

**Independent Test**: Open Settings → Add Scan Root → verify OS file picker opens → select directory → verify scan starts.

---

### P1: Tooltips ⭐ MVP

**User Story**: As a developer, I want tooltips on "dirty", "missing", "clean", "healthy", "warning", "critical", and tracking indicators so that I understand each status without consulting docs.

**Issues**: #44

**Acceptance Criteria**:

1. WHEN the user hovers over any status badge (dirty, clean, missing) THEN the system SHALL display a tooltip explaining the status
2. WHEN the user hovers over a health severity indicator (healthy, warning, critical) THEN the system SHALL display a tooltip with the severity meaning and threshold
3. WHEN the user hovers over a liveness dot (once implemented) THEN the system SHALL display env name, last check time, and response time
4. WHEN the user hovers over a sidebar navigation icon THEN the system SHALL display the page name as a tooltip
5. WHEN the user hovers over a tracking indicator (ahead/behind) THEN the system SHALL display upstream relationship context
6. WHEN a tooltip is displayed THEN it SHALL appear after a short delay (200–300ms) and dismiss on mouse-out

**Independent Test**: Hover over every status indicator on Dashboard; verify all show descriptive tooltips.

---

### P1: Configurable Pagination ⭐ MVP

**User Story**: As a developer with a large workspace, I want configurable pagination (10/25/50/100 per page) on the Dashboard and Changes tables so that the UI stays responsive.

**Issues**: #45

**Acceptance Criteria**:

1. WHEN the Dashboard table has more rows than the configured page size THEN the system SHALL paginate with prev/next controls
2. WHEN the Changes table has more rows than the configured page size THEN the system SHALL paginate by individual entries (flat), not by groups — group headings repeat across page breaks if needed (D102)
3. WHEN the user changes page size THEN the system SHALL persist the preference in Config as `page_size`
4. WHEN the user opens the page size selector THEN the options SHALL be 10, 25, 50, and 100 with default 25
5. WHEN pagination is active THEN the system SHALL display "Showing X–Y of Z" count
6. WHEN the data has fewer rows than page size THEN the pagination controls SHALL be hidden

**Independent Test**: Add 30+ repos → Dashboard shows 25 (default) with pagination → change to 10 → verify persistence after restart.

---

### P1: Unregister Repository ⭐ MVP

**User Story**: As a developer, I want to unregister a Repository from Gitty tracking without deleting it from disk so that I can stop tracking repos I no longer care about.

**Issues**: #46

**Acceptance Criteria**:

1. WHEN the user triggers unregister on a Repository THEN the system SHALL display a confirmation dialog
2. WHEN the confirmation dialog is shown THEN it SHALL warn that Group assignments, Tags, and Liveness configuration will be lost
3. WHEN the user confirms THEN the system SHALL remove the Repository from the Config registry
4. WHEN the user confirms THEN the system SHALL NOT modify the git repository on disk
5. WHEN a Repository is unregistered THEN it SHALL disappear from the Dashboard, sidebar, and all views
6. WHEN the user rescans the Scan Root THEN the unregistered repository SHALL be re-discoverable as a new Repository

**Independent Test**: Unregister a repo → verify it disappears from Dashboard → rescan → verify it reappears without prior Group/Tags.

---

### P2: Top Bar Redesign

**User Story**: As a developer, I want a workspace health indicator always visible in the top bar, a global search, and a proper SVG notification bell so that I can spot degradation and navigate quickly from any page.

**Issues**: #52 (depends on #42)

**Acceptance Criteria**:

1. WHEN the top bar is rendered THEN it SHALL display: `[Mascot 20px] Gitty ... [Health dot+score] [Search input] [Bell SVG]`
2. WHEN the top bar is rendered THEN the "Workspace Manager" subtitle and "Fetch All" button SHALL be removed
3. WHEN the health score changes THEN the top bar health indicator SHALL update in real-time (dot color + percentage)
4. WHEN the user types in the search input THEN the system SHALL return Repositories only — Group/Tag matching means "repos belonging to groups/tags whose name matches the query" (D103)
5. WHEN search results appear THEN clicking a result SHALL navigate to that Repository's detail page
6. WHEN the notification bell is rendered THEN it SHALL be a monochrome SVG icon matching the icon set from #42
7. WHEN unread notifications exist THEN the bell SHALL display a count badge

**Independent Test**: Navigate to Settings page → verify health score visible in top bar → type a repo name in search → verify results appear.

---

### P2: First-Run Onboarding

**User Story**: As a first-time user seeing an empty Dashboard, I want an onboarding card that guides me to add my first Scan Root so that I don't get stuck.

**Issues**: #53 (depends on #43)

**Acceptance Criteria**:

1. WHEN the Dashboard has zero Repositories THEN the system SHALL display an onboarding card replacing the existing empty state entirely (D101)
2. WHEN the onboarding card is displayed THEN it SHALL include a CTA button to open the native folder picker (from #43)
3. WHEN the user adds a Scan Root via the onboarding card THEN the system SHALL scan and populate the Dashboard
4. WHEN Repositories exist THEN the onboarding card SHALL NOT be displayed
5. WHEN the onboarding card is displayed THEN it SHALL briefly explain Gitty's purpose and the concept of Scan Roots

**Independent Test**: Fresh install → open Dashboard → verify onboarding card → add Scan Root via CTA → verify Dashboard populates.

---

## Edge Cases

- WHEN an SVG icon fails to load THEN the system SHALL fall back to a generic placeholder icon (not emoji)
- WHEN drag-and-drop receives multiple folders THEN the system SHALL add each as a separate Scan Root
- WHEN the user unregisters the last Repository THEN the Dashboard SHALL show the onboarding card
- WHEN pagination page size is changed while on page 3 THEN the system SHALL reset to page 1
- WHEN search input is cleared THEN the full Repository list SHALL be restored
- WHEN the native folder picker is cancelled THEN no action SHALL be taken

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| UX-01 | SVG Icon Set (monochrome SVGs) | #42 | P1 | |
| UX-02 | SVG Icon Set (default color muted) | #42 | P1 | |
| UX-03 | SVG Icon Set (hover brightens) | #42 | P1 | |
| UX-04 | SVG Icon Set (single library) | #42 | P1 | |
| UX-05 | SVG Icon Set (sidebar SVGs) | #42 | P1 | |
| UX-05b | SVG Icon Set (notification bell) | #42 | P1 | |
| UX-06 | Folder Picker (native dialog) | #43 | P1 | |
| UX-07 | Folder Picker (select adds root) | #43 | P1 | |
| UX-08 | Folder Picker (drag-and-drop) | #43 | P1 | |
| UX-09 | Folder Picker (drop zone visual) | #43 | P1 | |
| UX-10 | Folder Picker (manual fallback) | #43 | P1 | |
| UX-11 | Folder Picker (non-dir error) | #43 | P1 | |
| UX-12 | Tooltips (status badges) | #44 | P1 | |
| UX-13 | Tooltips (health severity) | #44 | P1 | |
| UX-14 | Tooltips (liveness dots) | #44 | P1 | |
| UX-15 | Tooltips (sidebar icons) | #44 | P1 | |
| UX-16 | Tooltips (tracking indicators) | #44 | P1 | |
| UX-17 | Tooltips (delay + dismiss) | #44 | P1 | |
| UX-18 | Pagination (Dashboard) | #45 | P1 | |
| UX-19 | Pagination (Changes) | #45 | P1 | |
| UX-20 | Pagination (persist page_size) | #45 | P1 | |
| UX-21 | Pagination (size options) | #45 | P1 | |
| UX-22 | Pagination (showing count) | #45 | P1 | |
| UX-23 | Pagination (auto-hide) | #45 | P1 | |
| UX-24 | Unregister (confirmation) | #46 | P1 | |
| UX-25 | Unregister (loss warning) | #46 | P1 | |
| UX-26 | Unregister (remove from config) | #46 | P1 | |
| UX-27 | Unregister (no disk change) | #46 | P1 | |
| UX-28 | Unregister (disappears from views) | #46 | P1 | |
| UX-29 | Unregister (re-discoverable) | #46 | P1 | |
| UX-30 | Top Bar (layout) | #52 | P2 | |
| UX-31 | Top Bar (remove subtitle + fetch) | #52 | P2 | |
| UX-32 | Top Bar (live health indicator) | #52 | P2 | |
| UX-33 | Top Bar (search filter) | #52 | P2 | |
| UX-34 | Top Bar (search navigation) | #52 | P2 | |
| UX-35 | Top Bar (SVG bell) | #52 | P2 | |
| UX-36 | Top Bar (unread badge) | #52 | P2 | |
| UX-37 | Onboarding (empty state card) | #53 | P2 | |
| UX-38 | Onboarding (folder picker CTA) | #53 | P2 | |
| UX-39 | Onboarding (scan after add) | #53 | P2 | |
| UX-40 | Onboarding (hide when repos exist) | #53 | P2 | |
| UX-41 | Onboarding (explanatory text) | #53 | P2 | |

**Coverage:** 41 requirements

---

## Success Criteria

- [ ] Zero emoji or Unicode icons remain in the UI
- [ ] Scan Root can be added via native OS folder picker and drag-and-drop
- [ ] Every status indicator shows a descriptive tooltip on hover
- [ ] Dashboard with 50+ repos paginates correctly with configurable page sizes
- [ ] Repository can be unregistered and re-discovered via rescan
- [ ] Top bar shows live health score and functional search from any page
- [ ] Empty Dashboard shows onboarding card with folder picker CTA
