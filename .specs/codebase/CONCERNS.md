# Concerns

## Active

_No active concerns._

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

**Resolution:** 165 tests total across the workspace.

### FE-1: Scaffold UI doesn't match design system

**Resolution:** Full design system implemented in M3 with CSS custom properties from DESIGN.md.

### FE-2: No ESLint or Prettier configured

**Resolution:** ESLint flat config + Prettier + Husky pre-commit hook.

### CLIPPY-1: Derivable Default impl in SchedulerConfig

**Resolution:** Replaced manual `impl Default for SchedulerConfig` with `#[derive(Default)]`.
