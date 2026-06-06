# M6 Sub-feature: Polish — Specification

## Problem Statement

The frontend uses a mix of CSS design tokens and hardcoded values (123 hardcoded `font-size` declarations, inline hex colors). Fonts load via Google Fonts CDN which is silently blocked by the Tauri CSP in production builds. Errors are displayed as raw strings with no severity distinction, no auto-dismiss for transient failures, and no recovery guidance. Git Operations have error classification but no retry logic — transient network errors require manual re-runs.

## Goals

- [x] Every visual property in Svelte components references a CSS design token (no hardcoded px/hex)
- [x] Fonts self-hosted and CSP-clean, working fully offline
- [x] Toast system for transient errors with auto-dismiss; inline contextual display for persistent errors
- [x] AppError DTO includes recovery hints populated from error classification
- [x] Per-step retry configuration on Macro definitions for Git Operations with Network errors

## Out of Scope

| Feature | Reason |
| --- | --- |
| Timeline pastel colors | No agent timeline in Gitty (D75) |
| Dark mode | Not in DESIGN.md; single theme for v1 |
| Retry for Shell Commands | CONTEXT.md: Shell Commands never auto-retried |
| Retry for non-Network errors | D76: only Network errors are safely retryable |
| Animation/transition system | DESIGN.md: animation timings out of scope |

---

## User Stories

### P1: Self-Host Fonts ⭐ MVP

**User Story**: As a developer, I want the app to load fonts without a network connection so that it works offline and doesn't violate the CSP policy.

**Acceptance Criteria**:

1. WHEN the app loads THEN system SHALL render text in Inter (400, 500, 600) and code in JetBrains Mono (400, 500)
2. WHEN the app loads without network THEN system SHALL still render the correct fonts (self-hosted)
3. WHEN the CSP is enforced THEN system SHALL not generate any CSP violation errors for font loading
4. WHEN `global.css` is inspected THEN it SHALL use `@font-face` declarations pointing to local files, not CDN URLs

**Independent Test**: Build the Tauri app; disconnect network; launch; verify Inter and JetBrains Mono render correctly.

---

### P1: Design System Token Completion ⭐ MVP

**User Story**: As a developer, I want every UI property to reference the design system so that visual changes are consistent and centralized.

**Acceptance Criteria**:

1. WHEN `tokens.css` is inspected THEN it SHALL contain every DESIGN.md token except timeline pastels: all colors, all spacing (including `--space-section: 80px`), all radii, and typography composite custom properties
2. WHEN any Svelte component is inspected THEN it SHALL not contain hardcoded hex color values — all colors reference `var(--color-*)`
3. WHEN any Svelte component is inspected THEN it SHALL use spacing tokens (`var(--space-*)`) for padding/margin/gap instead of raw pixel values, unless the value doesn't correspond to a token
4. WHEN any Svelte component is inspected THEN it SHALL use typography-related tokens or the global reset values for font-size/weight/letter-spacing

**Independent Test**: `grep -r '#[0-9a-f]' src/**/*.svelte` returns zero matches outside of comments or data attributes.

---

### P1: Toast Error System ⭐ MVP

**User Story**: As a developer, I want transient errors (network, lock contention) to show as auto-dismissing toasts so that they inform me without blocking the UI, while persistent errors (config corruption, git not found) display inline with recovery suggestions.

**Acceptance Criteria**:

1. WHEN a transient error occurs (error codes: `git_error` with Network category, `lock_contention`) THEN system SHALL display an auto-dismissing toast (5 seconds) at the top-right of the viewport
2. WHEN a persistent error occurs (error codes: `config_error`, `git_error` with non-Network category, `io_error`) THEN system SHALL display the error inline in the current page context with a recovery hint
3. WHEN a toast is displayed THEN it SHALL show the error message and a dismiss button
4. WHEN multiple toasts fire in quick succession THEN system SHALL stack them vertically (max 3 visible)
5. WHEN the user dismisses a toast THEN it SHALL animate out immediately

**Independent Test**: Trigger a network error via fetch on a non-existent remote; verify toast appears and auto-dismisses after 5 seconds.

---

### P1: AppError Recovery Hints ⭐ MVP

**User Story**: As a developer, I want error messages to include actionable recovery hints so that I know how to fix problems without searching documentation.

**Acceptance Criteria**:

1. WHEN a `CoreError::GitNotFound` is returned THEN the AppError SHALL include hint "Install Git and ensure it is in your PATH, then restart Gitty."
2. WHEN a `CoreError::LockContention` is returned THEN the AppError SHALL include hint "Another process is using this Repository. Wait for it to finish or check for stale locks."
3. WHEN a `CoreError::UnsupportedSchema` is returned THEN the AppError SHALL include hint "This config was created by a newer version of Gitty. Update Gitty to the latest version."
4. WHEN a `CoreError::PathNotFound` is returned THEN the AppError SHALL include hint "The path does not exist on disk. Check that it hasn't been moved or deleted."
5. WHEN a `CoreError::Io` with permission denied is returned THEN the AppError SHALL include hint "Permission denied. Check file permissions or run with elevated privileges."
6. WHEN any other error is returned THEN the AppError SHALL have `hint: None`
7. WHEN the frontend receives an error with a hint THEN it SHALL display the hint below the error message in muted text

**Independent Test**: Trigger a path-not-found error; verify the hint text appears in the UI.

---

### P2: Component Audit — Hardcoded Values

**User Story**: As a developer maintaining the codebase, I want all components to consistently use the design system so that changing a token updates the entire app.

**Acceptance Criteria**:

1. WHEN each Svelte component's `<style>` block is inspected THEN hardcoded `font-size`, `padding`, `margin`, `gap`, `border-radius`, `color`, and `background` values that correspond to design tokens SHALL be replaced with `var(--token)` references
2. WHEN `global.css` is inspected THEN shared utility classes (`.btn-*`, `.dialog-*`, `.badge-*`, etc.) SHALL exclusively use design tokens
3. WHEN new CSS properties don't map to existing tokens THEN they SHALL be documented as intentional exceptions (e.g., `2px` values below the `--space-xxs: 4px` minimum)

**Independent Test**: Audit all `*.svelte` files; verify zero hardcoded hex colors and minimal hardcoded pixel values.

---

### P2: Macro-Level Retry Configuration

**User Story**: As a developer, I want to configure retry behavior per Macro step so that transient network errors are automatically retried without me having to re-run the entire Macro.

**Acceptance Criteria**:

1. WHEN a Step is a Git Operation THEN it SHALL accept an optional `retry` config with `max_attempts` (default: no retry) and `backoff_seconds` (default: 2)
2. WHEN a Git Operation fails with `ErrorCategory::Network` AND retry is configured THEN the execution engine SHALL retry up to `max_attempts` times with exponential backoff (`backoff_seconds * 2^attempt`)
3. WHEN a Git Operation fails with a non-Network error AND retry is configured THEN the execution engine SHALL NOT retry
4. WHEN a Shell Command step has retry configured THEN the execution engine SHALL ignore it (Shell Commands never retry per CONTEXT.md)
5. WHEN retry attempts are exhausted THEN the step SHALL report the last error
6. WHEN the CLI defines a step with retry THEN the syntax SHALL be `fetch:retry=3` or `pull:retry=3:backoff=5`
7. WHEN the Macro editor in the GUI shows a Git Op step THEN it SHALL display an optional "Retry on network error" checkbox with max attempts input

**Independent Test**: Define a macro with `fetch:retry=3`; simulate network error; verify 3 attempts with increasing delays.

---

## Edge Cases

- WHEN a toast is displayed and the user navigates to another page THEN the toast SHALL persist (it's viewport-level, not page-level)
- WHEN an error has both a message and a hint THEN the hint SHALL be displayed below the message, never replacing it
- WHEN retry is configured with `max_attempts: 0` THEN it SHALL be treated as no retry (same as omitting the field)
- WHEN retry backoff would exceed 60 seconds THEN it SHALL cap at 60 seconds
- WHEN the last retry attempt succeeds THEN the step SHALL report success (not the prior failures)
- WHEN font files are missing from `static/fonts/` THEN the CSS fallback chain (`system-ui`, `Helvetica Neue`, etc.) SHALL activate

---

## Requirement Traceability

| Requirement ID | Story | Priority | Status |
| --- | --- | --- | --- |
| FONT-01 | Self-Host Fonts (render correct fonts) | P1 | Done |
| FONT-02 | Self-Host Fonts (offline) | P1 | Done |
| FONT-03 | Self-Host Fonts (CSP clean) | P1 | Done |
| FONT-04 | Self-Host Fonts (@font-face) | P1 | Done |
| TOKEN-01 | Token Completion (tokens.css complete) | P1 | Done |
| TOKEN-02 | Token Completion (no hardcoded hex) | P1 | Done |
| TOKEN-03 | Token Completion (spacing tokens) | P1 | Done |
| TOKEN-04 | Token Completion (typography tokens) | P1 | Done |
| TOAST-01 | Toast System (transient auto-dismiss) | P1 | Done |
| TOAST-02 | Toast System (persistent inline) | P1 | Done |
| TOAST-03 | Toast System (dismiss button) | P1 | Done |
| TOAST-04 | Toast System (stacking) | P1 | Done |
| TOAST-05 | Toast System (dismiss animation) | P1 | Done |
| HINT-01 | Recovery Hints (GitNotFound) | P1 | Done |
| HINT-02 | Recovery Hints (LockContention) | P1 | Done |
| HINT-03 | Recovery Hints (UnsupportedSchema) | P1 | Done |
| HINT-04 | Recovery Hints (PathNotFound) | P1 | Done |
| HINT-05 | Recovery Hints (Permission denied) | P1 | Done |
| HINT-06 | Recovery Hints (default None) | P1 | Done |
| HINT-07 | Recovery Hints (frontend display) | P1 | Done |
| AUDIT-01 | Component Audit (replace hardcoded) | P2 | Done |
| AUDIT-02 | Component Audit (global.css tokens) | P2 | Done |
| AUDIT-03 | Component Audit (document exceptions) | P2 | Done |
| RETRY-01 | Macro Retry (Step config) | P2 | Done |
| RETRY-02 | Macro Retry (Network only) | P2 | Done |
| RETRY-03 | Macro Retry (non-Network skip) | P2 | Done |
| RETRY-04 | Macro Retry (Shell ignored) | P2 | Done |
| RETRY-05 | Macro Retry (exhausted report) | P2 | Done |
| RETRY-06 | Macro Retry (CLI syntax) | P2 | Done |
| RETRY-07 | Macro Retry (GUI editor) | P2 | Done |

**Coverage:** 30 requirements, 30 verified

---

## Success Criteria

- [x] App launches and renders correct fonts with no network and zero CSP violations
- [x] `grep -rn` for hardcoded hex colors in `*.svelte` returns zero results
- [x] Network errors on fetch/pull show toast that auto-dismisses
- [x] Config errors show inline with recovery hint text
- [x] Macro with retry=3 retries network failures 3 times before reporting failure
