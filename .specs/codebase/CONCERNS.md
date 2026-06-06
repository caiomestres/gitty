# Concerns

**Analyzed:** 2026-06-06

## Active

### FE-3: Limited frontend test coverage

**Severity:** Medium
**Location:** `src/` (entire frontend)
**Evidence:** Only `src/lib/smoke.test.ts` exists — a single smoke test. 20 components and 7 routes have zero test coverage.
**Impact:** UI regressions go undetected until manual testing. Error handling paths (toast routing, hint display) are not verified.
**Fix approach:** Add Vitest component tests for critical paths (error handling, macro editor, scheduler settings). Consider Playwright for E2E flows.

### FE-4: No E2E test infrastructure

**Severity:** Low (pre-release)
**Location:** N/A
**Evidence:** Tauri 2 supports WebDriver testing but nothing is configured.
**Impact:** Full user flows (scan → configure → execute macro → verify health) cannot be automatically verified.
**Fix approach:** Add `@tauri-apps/driver` or Playwright for critical user journeys before v1 release.

## Resolved

### CODE-1: CLI main.rs at 1012 lines -> decomposed

**Resolution:** Extracted 8 command handler modules into `crates/gitty-cli/src/commands/` (workspace, group, tag, filter, macros, health, scheduler, notification). Shared helper `resolve_group_id` in `commands/mod.rs`. `main.rs` reduced to ~220 lines (CLI struct definitions + dispatch).

### CODE-2: name_from_path duplicated Repository::display_name()

**Resolution:** Removed `name_from_path()` from `src-tauri/src/commands/mod.rs`. The one remaining usage in `batch_to_dto()` extracts the name inline from `PathBuf` (not a `Repository`, so `display_name()` is unavailable in that context).

### CODE-3: Health/Settings pages used $effect instead of onMount

**Resolution:** Replaced `$effect(() => { loadHealth(); })` with `onMount(() => { loadHealth(); })` in `src/routes/health/+page.svelte`. Same fix applied to `src/routes/settings/+page.svelte` for `loadScanRoots()`, `loadSchedulerConfig()`, and `loadNotifConfig()`. Aligns with D64.

### CODE-4: Dead variable in scheduler runner

**Resolution:** Renamed `repos` to `active_repos` and used it directly for both macro execution and health evaluation, eliminating the redundant `Selection::All.resolve()` call and unnecessary `.cloned()`.

### ARCH-1: Single crate vs. planned workspace

**Resolution:** Migrated to Cargo workspace with 3 members: `gitty-core`, `gitty-cli`, `src-tauri`.

### SEC-1: CSP not configured

**Resolution:** CSP configured in `src-tauri/tauri.conf.json`.

### TEST-1: No tests exist

**Resolution:** 191 tests total across the workspace.

### FE-1: Scaffold UI doesn't match design system

**Resolution:** Full design system implemented in M3 with CSS custom properties from DESIGN.md. M6 completed font self-hosting (D74) and token-complete audit (D73).

### FE-2: No ESLint or Prettier configured

**Resolution:** ESLint flat config + Prettier + Husky pre-commit hook. CI enforces.

### CLIPPY-1: Derivable Default impl in SchedulerConfig

**Resolution:** Replaced manual `impl Default for SchedulerConfig` with `#[derive(Default)]`.
