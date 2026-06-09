# M7: Liveness Monitoring — Specification

## Problem Statement

The Health system evaluates code hygiene (staleness, divergence, dirty tree, detached HEAD) but has no concept of service liveness. Developers managing deployed services alongside their repositories cannot see whether those services are reachable. They must switch to separate monitoring tools to check if dev/qa/prd environments are up, fragmenting their workflow.

## Goals

- [ ] Per-Repository, per-Environment HTTP endpoint probing
- [ ] Independent liveness status that does not affect Workspace Health score
- [ ] Visual liveness indicators (green/red/gray dots) on the Dashboard
- [ ] Convention-based endpoint auto-discovery from repository files
- [ ] Optional notifications on probe failure
- [ ] Scheduler integration for periodic probing

## Out of Scope

| Feature | Reason |
| --- | --- |
| Liveness history graphs / uptime % | Future enhancement (PRD) |
| Auto-discovery from live Kubernetes clusters | Only static file scanning (PRD) |
| CLI commands for liveness | GUI-first; CLI follow-up deferred |
| TCP/gRPC probes | HTTP GET only for v1 |
| Custom probe headers / auth | Simple health check; auth deferred |

---

## User Stories

### P1: Liveness Core Pipeline ⭐ MVP

**User Story**: As a developer managing deployed services, I want to configure HTTP health endpoints per Repository per Environment so that I can see at a glance whether my services are up.

**Issues**: #48

**Acceptance Criteria**:

1. WHEN a user adds an Environment to a Repository THEN it SHALL accept: name (string), url (string), health_path (string, default "/health"), enabled (bool), interval_seconds (u32, default 300)
2. WHEN a Repository has one or more Environments configured THEN the system SHALL display green/red/gray dots in the Dashboard table
3. WHEN a liveness probe executes THEN it SHALL perform an HTTP GET to `{url}{health_path}` with a 10-second timeout
4. WHEN the probe receives a 2xx response THEN the result SHALL be Up
5. WHEN the probe receives a non-2xx response, timeout, or connection error THEN the result SHALL be Down
6. WHEN the Scheduler runs THEN it SHALL execute liveness probes at the configured interval for each enabled Environment
7. WHEN a liveness dot is gray THEN it SHALL indicate "never probed" (no result yet)
8. WHEN liveness status changes THEN it SHALL NOT affect the Workspace Health score (ADR-0010)
9. WHEN Environments are configured THEN they SHALL be stored in Config under `repositories.<uuid>.environments`

**Independent Test**: Add an Environment to a repo → trigger probe → verify green dot on Dashboard → stop the service → re-probe → verify red dot.

---

### P1: Environment CRUD ⭐ MVP

**User Story**: As a developer, I want to add multiple environments per repo (dev, qa, hml, prd) so that I can monitor all deployment targets from one place.

**Issues**: #48

**Acceptance Criteria**:

1. WHEN the user opens a Repository's detail page THEN an "Environments" section SHALL allow adding/editing/removing Environments
2. WHEN the user adds an Environment THEN the system SHALL validate that name is non-empty and url is a valid HTTP(S) URL
3. WHEN the user edits an Environment THEN changes SHALL be persisted to Config immediately
4. WHEN the user removes an Environment THEN it SHALL be removed from Config after confirmation
5. WHEN multiple Environments exist for a Repository THEN each SHALL be probed independently

**Independent Test**: Add dev + prd environments → verify both appear in detail page → remove one → verify removal persisted.

---

### P2: Convention-Based Endpoint Discovery

**User Story**: As a developer, I want Gitty to suggest likely health endpoints by scanning repo files so that I don't have to manually look up every URL.

**Issues**: #54 (depends on #48)

**Acceptance Criteria**:

1. WHEN the user opens Environment configuration THEN the system SHALL offer a "Discover endpoints" action
2. WHEN discovery runs THEN it SHALL scan `docker-compose.yml`, `Dockerfile`, `.env*`, `Procfile`, and Kubernetes manifests in the repository root
3. WHEN a port/host is found THEN the system SHALL suggest `http://localhost:{port}/health` as a candidate endpoint
4. WHEN suggestions are presented THEN the user SHALL confirm, edit, or dismiss each suggestion before it becomes an Environment
5. WHEN no discoverable files exist THEN the system SHALL display "No endpoints discovered" and allow manual entry
6. WHEN the user overrides a suggestion THEN the override SHALL take precedence and be persisted

**Independent Test**: Repo with `docker-compose.yml` exposing port 8080 → discover → verify suggestion of `http://localhost:8080/health`.

---

### P2: Liveness Notifications

**User Story**: As a developer, I want to optionally receive notifications when a liveness check fails so that I can catch outages even when the app isn't in focus.

**Issues**: #55 (depends on #48)

**Acceptance Criteria**:

1. WHEN `liveness.notify_on_failure` is true AND a probe transitions from Up to Down THEN the system SHALL generate a Notification
2. WHEN `liveness.notify_on_failure` is false THEN no liveness notifications SHALL be generated
3. WHEN a notification is generated THEN it SHALL use the existing Notification infrastructure (OS toast for critical, in-app panel)
4. WHEN multiple Environments go down simultaneously THEN the system SHALL aggregate into a single notification ("3 environments unreachable")
5. WHEN the default Config is loaded THEN `notify_on_failure` SHALL be false (opt-in)

**Independent Test**: Enable liveness notifications → stop a service → verify notification appears → disable → stop another → verify no notification.

---

## Edge Cases

- WHEN a URL has a trailing slash and health_path starts with slash THEN the system SHALL normalize (no double slash)
- WHEN the probe times out after 10s THEN it SHALL report Down (not hang the scheduler)
- WHEN a Repository is in Missing state THEN its liveness probes SHALL be skipped (consistent with Health check exclusion; D108)
- WHEN all Environments are disabled THEN no probes SHALL execute for that Repository
- WHEN the HTTP client encounters a redirect THEN it SHALL follow up to 3 redirects before reporting Down

---

## Domain Model

### New Types (gitty-core::liveness)

- `Environment { name, url, health_path, enabled, interval_seconds }` — per-Repository endpoint config
- `LivenessResult { environment_name, status: Up|Down, checked_at, response_time_ms?, error? }` — probe outcome
- `LivenessCheck` — orchestrator that probes all enabled Environments for a Repository

### Config Additions

```
"liveness": { "enabled": true, "default_interval_seconds": 300, "notify_on_failure": false }
"repositories.<uuid>.environments": [{ "name": "dev", "url": "...", "health_path": "/health", "enabled": true, "interval_seconds": 300 }]
```

### Dependencies

- `reqwest` added to `gitty-core` behind cargo feature flag `liveness` — avoids compile-time bloat for non-liveness builds (D94)
- CSP unchanged — probes run from Rust backend, not frontend webview
- Probe function uses trait/closure injection for testability (same pattern as `process.rs`)

### Scheduler Integration (D97)

Liveness runs as a separate tick inside the existing scheduler loop. `tick_with_config` runs the macro, then probes liveness — each on its own interval. The `SchedulerConfig.macro_id` field is unchanged; liveness intervals come from per-environment config.

### Config Layout (D99)

Global liveness preferences (`enabled`, `default_interval_seconds`, `notify_on_failure`) live at Config root. Per-repo `environments` live inside `workspace.repositories[]`. Per-env `interval_seconds` overrides the global default when set.

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| LIVE-01 | Core Pipeline (Environment schema) | #48 | P1 | |
| LIVE-02 | Core Pipeline (dashboard dots) | #48 | P1 | |
| LIVE-03 | Core Pipeline (HTTP GET probe) | #48 | P1 | |
| LIVE-04 | Core Pipeline (Up on 2xx) | #48 | P1 | |
| LIVE-05 | Core Pipeline (Down on failure) | #48 | P1 | |
| LIVE-06 | Core Pipeline (scheduler integration) | #48 | P1 | |
| LIVE-07 | Core Pipeline (gray = never probed) | #48 | P1 | |
| LIVE-08 | Core Pipeline (independent from health) | #48 | P1 | |
| LIVE-09 | Core Pipeline (Config storage) | #48 | P1 | |
| LIVE-10 | Environment CRUD (UI section) | #48 | P1 | |
| LIVE-11 | Environment CRUD (validation) | #48 | P1 | |
| LIVE-12 | Environment CRUD (persist on edit) | #48 | P1 | |
| LIVE-13 | Environment CRUD (remove with confirm) | #48 | P1 | |
| LIVE-14 | Environment CRUD (independent probing) | #48 | P1 | |
| LIVE-15 | Endpoint Discovery (action) | #54 | P2 | |
| LIVE-16 | Endpoint Discovery (file scanning) | #54 | P2 | |
| LIVE-17 | Endpoint Discovery (suggestions) | #54 | P2 | |
| LIVE-18 | Endpoint Discovery (user confirms) | #54 | P2 | |
| LIVE-19 | Endpoint Discovery (no files fallback) | #54 | P2 | |
| LIVE-20 | Endpoint Discovery (override persists) | #54 | P2 | |
| LIVE-21 | Notifications (opt-in on failure) | #55 | P2 | |
| LIVE-22 | Notifications (opt-out default) | #55 | P2 | |
| LIVE-23 | Notifications (existing infra) | #55 | P2 | |
| LIVE-24 | Notifications (aggregation) | #55 | P2 | |
| LIVE-25 | Notifications (default off) | #55 | P2 | |

**Coverage:** 25 requirements

---

## Success Criteria

- [ ] Environment can be added to a Repository with name, URL, and health path
- [ ] Dashboard shows green/red/gray liveness dots per Environment
- [ ] Scheduler probes endpoints at configured intervals
- [ ] Liveness status does not affect Workspace Health score
- [ ] Endpoint discovery suggests ports from docker-compose.yml
- [ ] Opt-in notifications fire on probe failure transitions
