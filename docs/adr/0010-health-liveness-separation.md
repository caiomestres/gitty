# ADR 0010: Health and Liveness as Independent Concepts

**Date:** 2026-06-08
**Status:** Accepted
**Context:** PRD-002 grill-with-docs session

## Context

Gitty's existing Health system evaluates code hygiene: staleness, divergence, dirty working tree, and detached HEAD. PRD-002 introduces service reachability monitoring — pinging HTTP endpoints to check if a repo's deployed service is up.

Both could be unified under a single "Health" umbrella with a composite score, or kept as independent concepts with separate indicators.

## Decision

**Keep Health Checks and Liveness Checks as independent, non-overlapping concepts.**

- **Health Check** evaluates code hygiene (staleness, divergence, dirty tree, detached HEAD). Produces healthy/warning/critical.
- **Liveness Check** probes a configured HTTP endpoint. Produces Up/Down.
- **Workspace Health score** is derived from Health Checks only. Liveness does not factor into it.
- In the UI, Health and Liveness are shown as separate indicators (side by side, not merged).

## Alternatives Considered

**Composite health** — merge liveness into the health score so a downed service degrades the repo's health. Rejected because:

1. Many repos have no deployed service — composite health would be meaningless or require special-casing.
2. Code hygiene and service availability have different audiences and response actions. A dirty tree is fixed by committing; a downed service is fixed by checking infrastructure.
3. A single composite score obscures which axis is actually problematic.

## Consequences

- Two separate status indicators per repo in the Dashboard UI.
- Two separate data models (`RepositoryHealth` and `LivenessResult`).
- The domain glossary must clearly distinguish between the two to prevent confusion.
- Future features (e.g., Dependency Map) can correlate the two without being constrained by a shared model.
