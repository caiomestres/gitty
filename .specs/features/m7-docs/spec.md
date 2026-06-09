# M7: Documentation — Specification

## Problem Statement

The docs site is minimal (4 pages, stock Material theme) and does not reflect the quality of the product. After implementing all M7 features, project documentation (README, CONTEXT.md, DESIGN.md) will be outdated.

## Goals

- [ ] Bruno-quality MkDocs Material site with custom typography and brand palette
- [ ] Expanded navigation (~20 pages in 5 sections)
- [ ] Final project documentation update reflecting all M7 changes

## Out of Scope

| Feature | Reason |
| --- | --- |
| Video tutorials | Text-based docs for v1 |
| API documentation (rustdoc) | Internal crate docs, not user-facing |
| Localization / i18n | English only (ADR-0003) |

---

## User Stories

### P2: Documentation Site Overhaul

**User Story**: As a developer evaluating Gitty, I want the docs site to look professional and be organized by task so that I can find what I need quickly.

**Issues**: #60

**Acceptance Criteria**:

1. WHEN the docs site is visited THEN it SHALL use custom CSS with Gitty brand palette (warm cream, Cursor Orange) and Inter + JetBrains Mono typography
2. WHEN the navigation is rendered THEN it SHALL contain ~20 pages organized in 5 sections: Introduction, Core Concepts, GUI Guide, CLI Reference, Advanced
3. WHEN the site is built THEN the following MkDocs Material features SHALL be enabled: `navigation.tabs`, `navigation.sections`, `navigation.expand`, `navigation.instant`, `search.suggest`, `search.highlight`, `content.code.copy`, `content.code.annotate`
4. WHEN a user searches THEN instant search with suggestions SHALL return relevant results
5. WHEN code blocks are displayed THEN they SHALL have copy buttons and support annotations

**Independent Test**: Run `mkdocs build` → verify zero warnings → open site → verify custom styling, 5-section nav, search, code copy buttons.

---

### P2: Final Documentation Update

**User Story**: As a developer, I want README, CONTEXT.md, DESIGN.md, and all docs to reflect the shipped product so that documentation is accurate.

**Issues**: #61 (depends on #42–#60)

**Acceptance Criteria**:

1. WHEN README.md is read THEN it SHALL reflect all M7 features (liveness, themes, activity log, etc.)
2. WHEN CONTEXT.md is read THEN all new domain terms SHALL be present and accurate
3. WHEN DESIGN.md is read THEN it SHALL document the theme system and all three bundled themes
4. WHEN any docs page references a feature THEN the description SHALL match the shipped implementation
5. WHEN the docs site is deployed THEN all screenshots SHALL show the current UI (not outdated M6 screenshots)

**Independent Test**: Read each doc file → cross-reference against shipped features → verify no stale references.

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| DOC-01 | Overhaul (custom CSS + brand) | #60 | P2 | |
| DOC-02 | Overhaul (5-section nav) | #60 | P2 | |
| DOC-03 | Overhaul (MkDocs features) | #60 | P2 | |
| DOC-04 | Overhaul (instant search) | #60 | P2 | |
| DOC-05 | Overhaul (code blocks) | #60 | P2 | |
| DOC-06 | Final (README updated) | #61 | P2 | |
| DOC-07 | Final (CONTEXT.md updated) | #61 | P2 | |
| DOC-08 | Final (DESIGN.md updated) | #61 | P2 | |
| DOC-09 | Final (docs match shipped) | #61 | P2 | |
| DOC-10 | Final (current screenshots) | #61 | P2 | |

**Coverage:** 10 requirements

---

## Success Criteria

- [ ] Docs site builds with zero warnings and uses Gitty brand styling
- [ ] Navigation contains ~20 pages in 5 organized sections
- [ ] README, CONTEXT.md, and DESIGN.md reflect all M7 features
- [ ] All screenshots show current UI
