# M6 Polish — Tasks

## Task Overview

| # | Task | Priority | Depends | Parallel |
|---|------|----------|---------|----------|
| T1 | Self-host fonts | P1 | — | [P] |
| T2 | Complete CSS tokens | P1 | — | [P] |
| T3 | AppError hint field (backend) | P1 | — | [P] |
| T4 | Toast store + component | P1 | — | [P] |
| T5 | Wire toast system into pages | P1 | T3, T4 | — |
| T6 | Component audit: replace hardcoded values | P2 | T2 | — |
| T7 | Retry config in core domain | P2 | — | [P] |
| T8 | Retry in execution engine | P2 | T7 | — |
| T9 | Retry CLI syntax | P2 | T7 | [P] with T8 |
| T10 | Retry GUI editor | P2 | T7 | [P] with T8 |

---

## T1: Self-Host Fonts

**What**: Download Inter (400/500/600) and JetBrains Mono (400/500) as woff2 files into `static/fonts/`. Replace the Google Fonts CDN `@import` in `global.css` with `@font-face` declarations.

**Where**:
- `static/fonts/` (new directory, 5 woff2 files)
- `src/lib/styles/global.css` (replace `@import url(...)` with `@font-face`)

**Done when**:
- 5 woff2 files exist in `static/fonts/`
- `global.css` has `@font-face` declarations, no CDN `@import`
- App renders Inter and JetBrains Mono offline
- No CSP violations in browser console

**Reqs**: FONT-01 through FONT-04

**Gate**: `npm run build` (verifies static assets included)

**Status**: Done

---

## T2: Complete CSS Tokens

**What**: Add missing DESIGN.md tokens to `tokens.css`: `--space-section`, `--color-warning`, and typography utility classes in `global.css`.

**Where**:
- `src/lib/styles/tokens.css`
- `src/lib/styles/global.css`

**Done when**:
- `tokens.css` has `--space-section: 80px` and `--color-warning: #c08532`
- `global.css` has typography utility classes (`.text-display-lg` through `.text-caption-up`)
- Existing components still render correctly

**Reqs**: TOKEN-01

**Gate**: `npm run check`

**Status**: Done

---

## T3: AppError Hint Field (Backend)

**What**: Add `hint: Option<String>` to `AppError` and populate it in the `From<CoreError>` implementation per the design doc's hint mapping table.

**Where**:
- `src-tauri/src/error.rs`

**Done when**:
- `AppError` has `pub hint: Option<String>`
- `From<CoreError>` populates hint for: `GitNotFound`, `LockContention`, `UnsupportedSchema`, `PathNotFound`, `Io` (permission denied), `NoConfigDir`
- All other variants produce `hint: None`
- Existing error mapping tests updated + new hint tests added

**Reqs**: HINT-01 through HINT-06

**Tests**: Build test-first. Add unit tests for each hint mapping.

**Gate**: `cargo test -p gitty-tauri && cargo clippy -- -D warnings`

**Status**: Done

---

## T4: Toast Store + Component

**What**: Create a toast notification system: `ToastStore` (Svelte 5 runes) + `ToastContainer` + `Toast` components. Mount `ToastContainer` in `AppShell`.

**Where**:
- `src/lib/stores/toast.ts` (new)
- `src/lib/components/ToastContainer.svelte` (new)
- `src/lib/components/AppShell.svelte` (mount ToastContainer)

**Done when**:
- `addToast()` and `dismissToast()` functions exported from store
- `ToastContainer` renders at fixed top-right, z-index 200
- Toasts show message + optional hint + dismiss button
- Auto-dismiss after configurable ms (default 5000)
- Max 3 visible, stacked vertically
- Styling follows DESIGN.md (surface-card bg, hairline border, radius-lg)

**Reqs**: TOAST-01 through TOAST-05

**Gate**: `npm run check`

**Status**: Done

---

## T5: Wire Toast System Into Pages

**What**: Replace raw `errorMessage()` usage across all pages with `handleError()` that routes transient errors to toast and persistent errors to inline state. Update `ErrorDto` to include `hint`.

**Where**:
- `src/lib/types/workspace.ts` (update `ErrorDto`, add `handleError()`)
- All 6 page files + `MacroRunner.svelte` (replace `errorMessage()` calls)

**Depends on**: T3 (hint field exists in backend), T4 (toast system exists)

**Done when**:
- `ErrorDto` has `hint?: string`
- `handleError()` classifies errors as transient/persistent
- Transient errors → `addToast()`
- Persistent errors → page-local `error` state with hint displayed in muted text
- All pages and components updated

**Reqs**: HINT-07, TOAST-01, TOAST-02

**Gate**: `npm run check`

**Status**: Done

---

## T6: Component Audit — Replace Hardcoded Values

**What**: Systematic pass through every `*.svelte` file and `global.css` replacing hardcoded hex colors with `var(--color-*)`, hardcoded spacing with `var(--space-*)`, and hardcoded font-size with typography classes or tokens where applicable.

**Where**: All files in `src/lib/components/`, `src/routes/`, `src/lib/styles/global.css`

**Depends on**: T2 (tokens must be complete first)

**Done when**:
- Zero hardcoded hex colors in `*.svelte` `<style>` blocks
- Spacing values that match tokens use token references
- Font-size values that match typography tokens use token references or utility classes
- Exceptions documented as comments where no token applies
- Visual appearance unchanged

**Reqs**: AUDIT-01 through AUDIT-03, TOKEN-02 through TOKEN-04

**Gate**: `npm run check && npm run build`

**Status**: Done

---

## T7: Retry Config in Core Domain

**What**: Add `RetryConfig` struct and `retry` field to `Step` in `macro_def.rs`. Ensure serde round-trip works with `#[serde(default)]`.

**Where**:
- `crates/gitty-core/src/macro_def.rs`

**Done when**:
- `RetryConfig` struct with `max_attempts: u32` and `backoff_seconds: u64`
- `Step.retry: Option<RetryConfig>` with `#[serde(default)]`
- Existing config files load without error (backward compat)
- Serde round-trip test for Step with and without retry

**Reqs**: RETRY-01

**Tests**: Build test-first. Serde serialization + backward compat test.

**Gate**: `cargo test -p gitty-core && cargo clippy -- -D warnings`

**Status**: Done

---

## T8: Retry in Execution Engine

**What**: Implement `execute_git_op_with_retry` in `execution.rs` that wraps `execute_git_op` with retry logic for Network errors only.

**Where**:
- `crates/gitty-core/src/execution.rs`

**Depends on**: T7

**Done when**:
- `execute_git_op_with_retry` retries on `ErrorCategory::Network` up to `max_attempts` times
- Exponential backoff: `backoff_seconds * 2^attempt`, capped at 60s
- Non-Network errors fail immediately (no retry)
- Shell Command steps bypass retry entirely
- Last attempt's result is returned on exhaustion
- Successful retry returns success

**Reqs**: RETRY-02 through RETRY-05

**Tests**: Unit test with mock git output simulating Network then success; Network exhaustion; non-Network no retry.

**Gate**: `cargo test -p gitty-core && cargo clippy -- -D warnings`

**Status**: Done

---

## T9: Retry CLI Syntax

**What**: Extend step parsing in `gitty-cli` to support `fetch:retry=3` and `pull:retry=3:backoff=5` syntax.

**Where**:
- `crates/gitty-cli/src/commands/macro_cmd.rs` (or wherever step parsing lives)

**Depends on**: T7

**Done when**:
- `fetch:retry=3` parses to `RetryConfig { max_attempts: 3, backoff_seconds: 2 }`
- `pull:retry=3:backoff=5` parses to `RetryConfig { max_attempts: 3, backoff_seconds: 5 }`
- `fetch` alone parses to `retry: None` (backward compat)
- Shell steps ignore retry params if provided (warning printed)
- `gitty macro show` displays retry config when present

**Reqs**: RETRY-06

**Gate**: `cargo test -p gitty-cli && cargo clippy -- -D warnings`

**Status**: Done

---

## T10: Retry GUI Editor

**What**: Add retry config UI to `MacroEditor.svelte` for Git Op steps: checkbox "Retry on network error" + max attempts input.

**Where**:
- `src/lib/components/MacroEditor.svelte`
- `src/lib/types/workspace.ts` (update `StepDto`)

**Depends on**: T7

**Done when**:
- Git Op steps show "Retry on network error" checkbox
- When checked, number input for max attempts (1-10, default 3) appears
- Shell Command steps do not show retry option
- Retry config included in `StepDto` sent to backend
- Existing macros without retry load correctly

**Reqs**: RETRY-07

**Gate**: `npm run check`

**Status**: Done
