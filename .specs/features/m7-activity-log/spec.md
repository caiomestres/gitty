# M7: Activity Log — Specification

## Problem Statement

Users have no operational visibility into what Gitty does. Scans, fetches, macro runs, scheduler actions, and config changes happen silently. When something goes wrong or a user wonders "what happened while I was away", there is no history to consult.

## Goals

- [ ] Timestamped record of all Gitty operations
- [ ] Filterable sidebar page for browsing history
- [ ] Bounded ring buffer storage with configurable limit

## Out of Scope

| Feature | Reason |
| --- | --- |
| Log export (CSV/JSON) | Future enhancement |
| Log streaming / real-time tail | Not in PRD |
| External log aggregation | Desktop-only tool |

---

## User Stories

### P1: Activity Log Core ⭐ MVP

**User Story**: As a developer, I want to see a chronological Activity Log of all Gitty operations so that I can understand what happened and when.

**Issues**: #47

**Acceptance Criteria**:

1. WHEN an operation executes (scan, fetch, pull, macro run, scheduler run, liveness check, health evaluation, config change) THEN the system SHALL append an entry to the Activity Log
2. WHEN an entry is created THEN it SHALL contain: timestamp, operation type, target (repo name/id if applicable), details, duration_ms (if applicable), and error (if applicable)
3. WHEN the Activity Log page is opened THEN entries SHALL be displayed in reverse chronological order (newest first)
4. WHEN the user filters by operation type THEN only matching entries SHALL be shown
5. WHEN the user filters by repo name THEN only entries targeting that Repository SHALL be shown
6. WHEN the user filters by date range THEN only entries within the range SHALL be shown
7. WHEN the log exceeds the configured limit (default 1000) THEN the oldest entries SHALL be evicted (ring buffer)
8. WHEN the user changes `activity_log_limit` in Config THEN the limit SHALL take effect on the next append
9. WHEN the Activity Log is stored THEN it SHALL be in `activity.json` alongside `config.json` (not inside Config)

**Independent Test**: Trigger a scan → open Activity Log → verify scan entry with timestamp and duration → trigger 1001 operations → verify oldest evicted.

---

### P1: Activity Log Page ⭐ MVP

**User Story**: As a developer, I want to filter the Activity Log by operation type, repo name, and date range so that I can find specific events quickly.

**Issues**: #47

**Acceptance Criteria**:

1. WHEN the sidebar is rendered THEN "Activity" SHALL appear between "Changes" and "Groups"
2. WHEN the Activity page loads THEN it SHALL display a filterable table of log entries
3. WHEN filters are applied THEN they SHALL work combinatorially (type AND repo AND date)
4. WHEN no entries match the filters THEN the system SHALL display an empty state message

**Independent Test**: Navigate to Activity → apply filters → verify table updates.

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| ALOG-01 | Core (append on operation) | #47 | P1 | |
| ALOG-02 | Core (entry schema) | #47 | P1 | |
| ALOG-03 | Core (reverse chronological) | #47 | P1 | |
| ALOG-04 | Core (filter by type) | #47 | P1 | |
| ALOG-05 | Core (filter by repo) | #47 | P1 | |
| ALOG-06 | Core (filter by date) | #47 | P1 | |
| ALOG-07 | Core (ring buffer eviction) | #47 | P1 | |
| ALOG-08 | Core (configurable limit) | #47 | P1 | |
| ALOG-09 | Core (activity.json storage) | #47 | P1 | |
| ALOG-10 | Page (sidebar placement) | #47 | P1 | |
| ALOG-11 | Page (filterable table) | #47 | P1 | |
| ALOG-12 | Page (combinatorial filters) | #47 | P1 | |
| ALOG-13 | Page (empty state) | #47 | P1 | |

**Coverage:** 13 requirements

---

## Success Criteria

- [ ] Every major Gitty operation appends to the Activity Log
- [ ] Activity page shows filterable reverse-chronological entries
- [ ] Ring buffer evicts oldest entries beyond the configured limit
