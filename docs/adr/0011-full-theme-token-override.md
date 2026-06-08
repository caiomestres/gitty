# ADR 0011: Full Design Token Override per Theme

**Date:** 2026-06-08
**Status:** Accepted
**Context:** PRD-002 grill-with-docs session

## Context

Gitty is adding a theme system (Default, Dark, World Cup - Brasil). The question is how much each theme can customize: colors only, colors + typography, or the full set of design tokens (colors, typography, spacing, border radii).

## Decision

**Each theme is a complete design token override: colors, typography, spacing, and border radii all vary.**

- `DESIGN.md` becomes the "Default Theme" specification.
- Core layout principles (grid, max-width, component structure) are shared and not theme-specific.
- Each theme is implemented as a CSS file that redefines all `--color-*`, `--text-*`, `--space-*`, and `--radius-*` custom properties.
- Applied via a `data-theme` attribute on the root element.

## Alternatives Considered

**Colors only** — simplest implementation, but limits thematic expression. The "World Cup - Brasil" theme wants a more energetic/playful feel that requires different font weights and spacing. A dark theme often benefits from slightly different spacing to compensate for perceived density. Rejected as too restrictive.

**Colors + typography only** — a middle ground, but spacing and radii are cheap to override and have real impact on theme personality. No compelling reason to draw the line here.

## Consequences

- Each new theme requires a complete token set (~40+ CSS custom properties), not just a color palette.
- Theme authors (currently only us) must consider how spacing and radii interact with component layouts.
- Components must use token references exclusively — no hardcoded values anywhere.
- Testing expands: each theme must be visually verified across all pages to catch spacing/overflow issues.
- The existing `DESIGN.md` is restructured from "the design system" to "Default Theme specification + shared layout principles."
