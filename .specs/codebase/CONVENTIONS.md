# Code Conventions

## Naming Conventions

**Files (Svelte):**
SvelteKit route conventions — `+page.svelte`, `+layout.ts`
Examples: `src/routes/health/+page.svelte`, `src/routes/+layout.ts`

**Files (Rust):**
Standard Rust — `lib.rs`, `main.rs`, `mod.rs`, snake_case for modules
Examples: `src-tauri/src/commands/health.rs`, `crates/gitty-core/src/scheduler/runner.rs`

**Functions (Rust):**
snake_case
Examples: `fn evaluate_workspace()`, `pub fn should_run()`, `fn battery_state()`

**Functions (TypeScript):**
camelCase
Examples: `async function getWorkspaceHealth()`, `function handleRefresh()`

**Variables (Svelte 5):**
camelCase with rune declarations
Examples: `let health = $state<WorkspaceHealthDto | null>(null)`, `let loading = $state(true)`

**Constants (Rust):**
SCREAMING_SNAKE_CASE
Examples: `CURRENT_SCHEMA_VERSION`, `HEALTH_FILE`, `UNGROUPED_GROUP_NAME`, `FAVORITE_TAG`

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

**Rust:** Standard Rust type system
- Domain types derive `Serialize + Deserialize` for persistence and IPC
- `thiserror` for typed error variants in core
- `AppError` DTOs with named codes for frontend consumption
- No `unsafe` in domain code (only in daemon.rs for Windows process management via FFI)

## Error Handling

**Rust (Core):**
`CoreError` enum with `thiserror::Error` derive. Public API returns `Result<T, CoreError>`.

**Rust (CLI):**
`anyhow::Result<()>` at the CLI boundary. Wraps `CoreError` with `.context()` for user messages.

**Rust (Tauri):**
`Result<T, AppError>` where `AppError` has `code` and `message` fields. `From<CoreError>` maps every variant to a named code.

**TypeScript:**
`errorMessage(e)` helper extracts error messages. Try/catch in async handlers with `$state` error variables.

## Patterns

**Initial Data Loading (Frontend):**
Use `onMount` for one-time data loads (D64). Do NOT use `$effect` for side-effect-driven initial loads.
```typescript
onMount(() => { loadChanges(); });
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

**Repository Display Name:**
Use `Repository::display_name()` method (D67) — never duplicate the path-to-name extraction.

## Tooling Enforcement

- `svelte-check` for frontend type checking (`npm run check`)
- `cargo clippy -- -D warnings` for Rust linting (warnings treated as errors)
- `cargo fmt` / `cargo fmt --check` for Rust formatting
- ESLint flat config for TypeScript/Svelte linting (`npm run lint`)
- Prettier for TypeScript/Svelte formatting (`npm run format`)
- Husky pre-commit hook running lint-staged
