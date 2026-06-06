# Code Conventions

**Analyzed:** 2026-06-06

## Naming Conventions

**Files (Svelte):**
SvelteKit route conventions — `+page.svelte`, `+layout.ts`
Examples: `src/routes/health/+page.svelte`, `src/routes/+layout.ts`

**Files (Svelte components):**
PascalCase
Examples: `AppShell.svelte`, `MacroEditor.svelte`, `ToastContainer.svelte`, `RepoGroupSelect.svelte`

**Files (Rust):**
Standard Rust — `lib.rs`, `main.rs`, `mod.rs`, snake_case for modules
Examples: `src-tauri/src/commands/health.rs`, `crates/gitty-core/src/scheduler/runner.rs`

**Functions (Rust):**
snake_case
Examples: `fn evaluate_workspace()`, `pub fn should_run()`, `fn battery_state()`, `fn hint_for_core_error()`

**Functions (TypeScript):**
camelCase
Examples: `async function getWorkspaceHealth()`, `function handleRefresh()`, `function handleError()`

**Variables (Svelte 5):**
camelCase with rune declarations
Examples: `let health = $state<WorkspaceHealthDto | null>(null)`, `let loading = $state(true)`

**Constants (Rust):**
SCREAMING_SNAKE_CASE
Examples: `CURRENT_SCHEMA_VERSION`, `HEALTH_FILE`, `UNGROUPED_GROUP_NAME`, `FAVORITE_TAG`

**Constants (TypeScript):**
SCREAMING_SNAKE_CASE for arrays/objects, camelCase for simple values
Examples: `DAY_OPTIONS`, `nextId`

**Enums (Rust):**
PascalCase variants, serde `rename_all = "lowercase"` or `rename_all = "snake_case"` for JSON
Examples: `CheckSeverity::Critical`, `RepositoryState::Missing`, `NotificationTrigger::OnCritical`

## Code Organization

**Import/Dependency Declaration (TypeScript):**
Framework imports first (`@tauri-apps/api/core`), then Svelte imports, then local
```typescript
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";
import type { WorkspaceHealthDto } from "$lib/types/health";
import { handleError, success } from "$lib/utils/error-handling";
```

**Import/Dependency Declaration (Rust):**
Standard library → external crates → `crate::` local modules
```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::Result;
```

**File Structure (Svelte):**
`<script lang="ts">` → markup → `<style>` (standard Svelte ordering)

**File Structure (Rust modules):**
Section comments separate concerns:
```
// Data Models
// (structs, enums)
// Logic
// (functions, trait impls)
// Tests
#[cfg(test)] mod tests { ... }
```

**Tauri Command Modules:**
Each `src-tauri/src/commands/*.rs` follows: DTOs → conversion helpers → `#[tauri::command]` functions → tests

## Type Safety

**TypeScript:** Strict mode enabled (`"strict": true` in tsconfig.json)
- `allowJs` and `checkJs` both true
- `moduleResolution: "bundler"`
- DTO types in `src/lib/types/*.ts` mirror Rust DTOs exactly
- `ActionFeedback` interface for unified success/error UI state

**Rust:** Standard Rust type system
- Domain types derive `Serialize + Deserialize` for persistence and IPC
- `thiserror` for typed error variants in core
- `AppError` DTOs with named codes + recovery hints for frontend consumption
- No `unsafe` in domain code (only in daemon.rs for Windows process management via FFI)

## Error Handling

**Rust (Core):**
`CoreError` enum with `thiserror::Error` derive. Public API returns `Result<T, CoreError>`.

**Rust (CLI):**
`anyhow::Result<()>` at the CLI boundary. Wraps `CoreError` with `.context()` for user messages.

**Rust (Tauri):**
`Result<T, AppError>` where `AppError` has `code`, `message`, `hint`, and `transient` fields. `From<CoreError>` maps every variant to a named code. `hint_for_core_error()` provides recovery suggestions. `is_transient()` classifies errors for UI routing.

**TypeScript:**
`handleError(e)` utility classifies errors: transient → routes to toast store and returns `null`; persistent → returns `ActionFeedback` for inline display with hint. Pages use `let feedback = $state<ActionFeedback | null>(null)` pattern.

## Patterns

**Initial Data Loading (Frontend):**
Use `onMount` for one-time data loads (D64). Do NOT use `$effect` for side-effect-driven initial loads.
```typescript
onMount(() => { loadChanges(); });
```

**Config Change Reactivity (Frontend):**
Use `$effect` with `onConfigChanged()` helper for reloading data when config changes externally.
```typescript
$effect(() => {
  return onConfigChanged(() => reload());
});
```

**Config Mutations (Tauri):**
Use `state.with_config_write()` — acquires Mutex, runs closure, auto-saves.
```rust
state.with_config_write(|config| {
    config.workspace.create_group(&name, parent)?;
    Ok(group_to_dto(...))
})
```

**Config Reads (Tauri):**
Use `state.config()` — returns `MutexGuard<Config>`.

**Error Display (Frontend):**
Use `handleError()` for consistent error routing, `success()` for feedback messages. `PageError` component for inline errors, `ToastContainer` for transient.

**Repository Display Name:**
Use `Repository::display_name()` method (D67) — never duplicate the path-to-name extraction.

## Tooling Enforcement

- `svelte-check` for frontend type checking (`npm run check`)
- `cargo clippy -- -D warnings` for Rust linting (warnings treated as errors)
- `cargo fmt` / `cargo fmt --check` for Rust formatting
- ESLint 9.x flat config for TypeScript/Svelte linting (`npm run lint`)
- Prettier for TypeScript/Svelte formatting (`npm run format`)
- Husky pre-commit hook running lint-staged
- Taskfile (`task` CLI): `task front` runs all frontend checks, `task back` runs all backend checks, `task` runs both
- GitHub Actions CI enforces all checks on push/PR to main
