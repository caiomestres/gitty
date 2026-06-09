# M7: Brand Identity — Specification

## Problem Statement

Gitty has no brand identity. The app uses a generic "G" placeholder in the sidebar, a default Tauri icon in the system tray and taskbar, and stock assets in the installer and docs. It is indistinguishable from any other unsigned desktop app.

## Goals

- [ ] Golden lion tamarin (mico-leao-dourado) mascot illustration
- [ ] Full asset matrix: PNG at 9 sizes, @2x for macOS, ICO, ICNS, SVG, tray icons, favicon, OG image
- [ ] Per-theme mascot color variants (Default, Dark, World Cup - Brasil)

## Out of Scope

| Feature | Reason |
| --- | --- |
| Animated mascot | Static illustration only for v1 |
| Mascot merchandise assets | Not app-relevant |
| User-customizable mascot | Fixed brand identity |

---

## User Stories

### P1: Mascot Artwork ✓ DONE

**User Story**: As a developer, I want to see a distinctive mascot icon in the system tray so that I can quickly identify Gitty among other running apps.

**Issues**: #51 (D106 — artwork complete)

**Status**: Artwork is done. SVGs (detailed + simplified) and full PNG matrix (16–1024px), @2x, ICO, tray icons (22/24px) are in `src-tauri/icons/`. Docs assets (favicon.ico, favicon.svg, OG image) are in `docs/assets/`.

**Acceptance Criteria**:

1. ✓ WHEN the mascot is generated THEN it SHALL depict a golden lion tamarin in a full-color illustration style
2. ✓ WHEN the mascot is rendered THEN it SHALL have two tiers: detailed illustration (128px+) and simplified silhouette (16–64px)
3. ✓ WHEN both tiers are compared THEN they SHALL share the same character pose (derived from one canonical image)
4. ✓ WHEN assets are exported THEN the full matrix SHALL include: PNG at 16/22/24/32/48/64/128/256/512/1024px, @2x for macOS, ICO (multi-res), SVG source
5. ✓ WHEN tray icons are exported THEN they SHALL include 22px and 24px variants
6. ✓ WHEN docs assets are exported THEN they SHALL include favicon.ico, favicon.svg, and OG image (1200x630)
7. ✓ WHEN assets are placed THEN app icons go in `src-tauri/icons/`, docs assets in `docs/assets/`

---

### P2: Per-Theme Mascot Variants

**User Story**: As a developer, I want the mascot to change color to match my selected theme so that the brand feels cohesive with my visual preference.

**Issues**: #59 (depends on #51, #49)

**Acceptance Criteria**:

1. WHEN the Default theme is active THEN the mascot SHALL use the canonical golden/orange palette
2. WHEN the Dark theme is active THEN the mascot SHALL use a luminous/lighter variant suitable for dark backgrounds
3. WHEN the World Cup - Brasil theme is active THEN the mascot SHALL use green/yellow/blue national colors
4. WHEN variants are produced THEN they SHALL be created by color-shifting the canonical illustration (identical silhouette guaranteed)
5. WHEN the theme changes THEN the mascot in the sidebar, About section, and onboarding card SHALL update to match

**Independent Test**: Switch between all 3 themes → verify mascot color changes → verify silhouette is identical across variants.

---

## Requirement Traceability

| Requirement ID | Story | Issue | Priority | Status |
| --- | --- | --- | --- | --- |
| BRAND-01 | Mascot (golden lion tamarin) | #51 | P1 | Done |
| BRAND-02 | Mascot (two rendering tiers) | #51 | P1 | Done |
| BRAND-03 | Mascot (same pose) | #51 | P1 | Done |
| BRAND-04 | Mascot (full asset matrix) | #51 | P1 | Done |
| BRAND-05 | Mascot (tray icons) | #51 | P1 | Done |
| BRAND-06 | Mascot (docs assets) | #51 | P1 | Done |
| BRAND-07 | Mascot (file locations) | #51 | P1 | Done |
| BRAND-08 | Variants (Default golden) | #59 | P2 | |
| BRAND-09 | Variants (Dark luminous) | #59 | P2 | |
| BRAND-10 | Variants (Brasil colors) | #59 | P2 | |
| BRAND-11 | Variants (color-shift method) | #59 | P2 | |
| BRAND-12 | Variants (theme-reactive) | #59 | P2 | |

**Coverage:** 12 requirements

---

## Success Criteria

- [x] Golden lion tamarin mascot renders correctly at all sizes (16px tray to 1024px installer)
- [x] Full asset matrix exported and placed in correct directories
- [ ] Per-theme color variants maintain identical silhouette
- [ ] Active theme's mascot variant displayed throughout the app
