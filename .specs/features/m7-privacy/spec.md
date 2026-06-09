# M7: Privacy Communication — Specification

## Problem Statement

Gitty's architecture is fully offline with zero data collection, but this is never communicated to users. The privacy-first nature is a competitive advantage that remains invisible.

## Goals

- [ ] In-app About section with privacy statement
- [ ] Docs site Privacy page
- [ ] Clear, trustworthy wording

---

## User Stories

### P1: Privacy Communication ⭐ MVP

**User Story**: As a developer, I want to know that Gitty collects zero data and makes no network calls except to my own remotes and endpoints I configure, so that I can trust it with my work.

**Issues**: #50

**Acceptance Criteria**:

1. WHEN the user opens Settings THEN an "About Gitty" section SHALL display the privacy statement
2. WHEN the privacy statement is rendered THEN it SHALL include: "Gitty is fully offline. Your data never leaves your machine. There are no accounts, no telemetry, no analytics, no cloud sync. Network calls are only made to your own Git remotes and to liveness endpoints you explicitly configure."
3. WHEN the docs site is visited THEN a dedicated "Privacy" page SHALL exist in the navigation
4. WHEN the Getting Started docs page is viewed THEN it SHALL include a callout banner about Gitty's privacy-first architecture
5. WHEN the About section is rendered THEN it SHALL also display the app version and a link to the GitHub repository

**Independent Test**: Open Settings → scroll to About → verify privacy statement text → visit docs → verify Privacy page exists.

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| PRIV-01 | Privacy (About section in Settings) | #50 | P1 | |
| PRIV-02 | Privacy (statement wording) | #50 | P1 | |
| PRIV-03 | Privacy (docs Privacy page) | #50 | P1 | |
| PRIV-04 | Privacy (Getting Started callout) | #50 | P1 | |
| PRIV-05 | Privacy (version + GitHub link) | #50 | P1 | |

**Coverage:** 5 requirements

---

## Success Criteria

- [ ] About section in Settings displays privacy statement and version
- [ ] Docs site has a dedicated Privacy page
- [ ] Getting Started page includes privacy callout banner
