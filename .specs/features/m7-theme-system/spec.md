# M7: Theme System — Specification

## Problem Statement

The app ships with a single warm-cream visual identity derived from DESIGN.md. There is no dark mode despite it being the most-requested missing feature for developer tools. The design system uses CSS custom properties but has no abstraction layer for swapping entire token sets. Users have no way to change the app's appearance.

## Goals

- [ ] Full design-token override infrastructure (colors, typography, spacing, radii per theme)
- [ ] Default theme extracted from current DESIGN.md values
- [ ] Dark theme with full token coverage
- [ ] World Cup - Brasil theme with national colors
- [ ] Theme switcher in Settings with visual preview cards
- [ ] Theme toggle accessible from bottom bar

## Out of Scope

| Feature | Reason |
| --- | --- |
| User-creatable custom themes | Only 3 bundled themes ship in v1 (PRD) |
| Multi-country World Cup themes | Only Brasil ships (PRD) |
| Per-component theme overrides | Token-level granularity only |
| System theme auto-detection | Manual selection for v1 |
| Theme import/export | Fixed bundled set |

---

## User Stories

### P1: Theme Infrastructure ⭐ MVP

**User Story**: As a developer, I want a theme system that overrides all design tokens so that switching themes changes the entire visual identity.

**Issues**: #49

**Acceptance Criteria**:

1. WHEN a theme is applied THEN it SHALL override all `--color-*`, `--text-*`, `--space-*`, and `--radius-*` CSS custom properties
2. WHEN themes are implemented THEN each theme SHALL be a separate CSS file redefining all token variables; the current `tokens.css` becomes `theme-default.css` — it IS the Default theme (D105)
3. WHEN a theme is activated THEN the system SHALL apply it via a `data-theme` attribute on the root HTML element; other themes layer via `[data-theme="dark"]` selectors
4. WHEN the Default theme is loaded THEN it SHALL reproduce the current DESIGN.md appearance exactly (zero visual regression)
5. WHEN the app starts THEN it SHALL read the theme preference from Config and apply it before first paint
6. WHEN no theme preference exists in Config THEN it SHALL default to "default"
7. WHEN shared layout principles (grid, max-width, component structure) are rendered THEN they SHALL NOT be theme-specific (ADR-0011)

**Independent Test**: Set theme to "dark" in Config → launch app → verify dark theme applied before any flash of default theme.

---

### P1: Dark Theme ⭐ MVP

**User Story**: As a developer, I want a dark theme so that the app matches my preference and reduces eye strain.

**Issues**: #56 (depends on #49)

**Acceptance Criteria**:

1. WHEN the Dark theme is active THEN the canvas background SHALL be dark (approximately `#1a1a2e` range)
2. WHEN the Dark theme is active THEN text SHALL use lighter colors for readability (contrast ratio ≥4.5:1 for body text)
3. WHEN the Dark theme is active THEN all status badges, dots, and interactive elements SHALL remain distinguishable
4. WHEN the Dark theme is active THEN borders and separators SHALL use subtle lighter shades (not invisible)
5. WHEN the Dark theme CSS file is inspected THEN it SHALL define every token that the Default theme defines (complete coverage) — spacing/radii values match Default; "complete override" means complete coverage, not everything differs (D107)

**Independent Test**: Switch to Dark theme → navigate every page → verify all text readable, all elements visible, no hardcoded light colors bleeding through.

---

### P2: World Cup - Brasil Theme

**User Story**: As a developer, I want a World Cup - Brasil theme so that I can celebrate with national colors.

**Issues**: #57 (depends on #49)

**Acceptance Criteria**:

1. WHEN the Brasil theme is active THEN the primary palette SHALL use green (#009c3b), yellow (#ffdf00), and blue (#002776)
2. WHEN the Brasil theme is active THEN the canvas background SHALL use one of the national colors as a base (or a derived neutral)
3. WHEN the Brasil theme is active THEN all text SHALL maintain ≥4.5:1 contrast ratio
4. WHEN the Brasil theme CSS file is inspected THEN it SHALL define every token that the Default theme defines (complete coverage)

**Independent Test**: Switch to Brasil theme → verify green/yellow/blue palette → verify all text readable across all pages.

---

### P1: Theme Switcher UI ⭐ MVP

**User Story**: As a developer, I want to see theme previews before switching so that I know what I'm selecting.

**Issues**: #58 (depends on #49)

**Acceptance Criteria**:

1. WHEN the user opens the Theme section in Settings THEN the system SHALL display preview cards for each available theme
2. WHEN a preview card is displayed THEN it SHALL show a miniature representation of the theme's color palette and typography
3. WHEN the user clicks a preview card THEN the theme SHALL be applied immediately (live preview)
4. WHEN a theme is selected THEN the preference SHALL be persisted in Config as `theme: "default" | "dark" | "world-cup-brasil"`
5. WHEN the bottom bar is rendered THEN it SHALL include a theme toggle for quick switching
6. WHEN the current theme is active THEN its preview card SHALL show a selected/active indicator

**Independent Test**: Open Settings → Theme section → click Dark preview card → verify theme applies → restart app → verify Dark persists.

---

## Edge Cases

- WHEN a component uses a CSS custom property that a theme doesn't define THEN the browser's fallback value SHALL apply (themes must be token-complete to avoid this)
- WHEN the user changes theme while a dialog is open THEN the dialog SHALL update to the new theme immediately
- WHEN the `theme` field in Config contains an unknown value THEN the system SHALL fall back to "default"
- WHEN switching themes rapidly THEN no flash of unstyled content SHALL occur

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| THEME-01 | Infrastructure (full token override) | #49 | P1 | |
| THEME-02 | Infrastructure (separate CSS files) | #49 | P1 | |
| THEME-03 | Infrastructure (data-theme attribute) | #49 | P1 | |
| THEME-04 | Infrastructure (Default = DESIGN.md) | #49 | P1 | |
| THEME-05 | Infrastructure (apply on startup) | #49 | P1 | |
| THEME-06 | Infrastructure (default fallback) | #49 | P1 | |
| THEME-07 | Infrastructure (layout not themed) | #49 | P1 | |
| THEME-08 | Dark (dark canvas) | #56 | P1 | |
| THEME-09 | Dark (text contrast) | #56 | P1 | |
| THEME-10 | Dark (elements distinguishable) | #56 | P1 | |
| THEME-11 | Dark (borders visible) | #56 | P1 | |
| THEME-12 | Dark (complete coverage) | #56 | P1 | |
| THEME-13 | Brasil (national palette) | #57 | P2 | |
| THEME-14 | Brasil (canvas base) | #57 | P2 | |
| THEME-15 | Brasil (text contrast) | #57 | P2 | |
| THEME-16 | Brasil (complete coverage) | #57 | P2 | |
| THEME-17 | Switcher (preview cards) | #58 | P1 | |
| THEME-18 | Switcher (miniature preview) | #58 | P1 | |
| THEME-19 | Switcher (click applies) | #58 | P1 | |
| THEME-20 | Switcher (persist preference) | #58 | P1 | |
| THEME-21 | Switcher (bottom bar toggle) | #58 | P1 | |
| THEME-22 | Switcher (active indicator) | #58 | P1 | |

**Coverage:** 22 requirements

---

## Success Criteria

- [ ] Three themes selectable: Default, Dark, World Cup - Brasil
- [ ] Default theme is pixel-identical to current DESIGN.md appearance
- [ ] Dark theme has full token coverage with ≥4.5:1 text contrast
- [ ] Theme switcher in Settings with visual preview cards
- [ ] Bottom bar provides quick theme toggle
- [ ] Theme preference persists across restarts
