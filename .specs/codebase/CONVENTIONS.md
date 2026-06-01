# Code Conventions

> Based on the scaffold code. Conventions will expand as feature code is written.

## Naming Conventions

**Files (Svelte):**
SvelteKit route conventions — `+page.svelte`, `+layout.ts`
Examples: `src/routes/+page.svelte`, `src/routes/+layout.ts`

**Files (Rust):**
Standard Rust — `lib.rs`, `main.rs`, snake_case for modules
Examples: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`

**Functions (Rust):**
snake_case
Examples: `fn greet(name: &str)`, `pub fn run()`

**Functions (TypeScript):**
camelCase
Examples: `async function greet(event: Event)`

**Variables (Svelte 5):**
camelCase with rune declarations
Examples: `let name = $state("")`, `let greetMsg = $state("")`

**Constants (Rust):**
Not yet established (no constants in scaffold)

## Code Organization

**Import/Dependency Declaration (TypeScript):**
Framework imports first (`@tauri-apps/api/core`), then local
```typescript
import { invoke } from "@tauri-apps/api/core";
```

**Import/Dependency Declaration (Rust):**
Not yet established — scaffold uses no explicit `use` statements beyond `tauri::command` attribute

**File Structure (Svelte):**
`<script>` → markup → `<style>` (standard Svelte ordering)

**File Structure (Rust):**
Attributes/macros → function definitions → `pub fn run()` app builder at bottom

## Type Safety

**TypeScript:** Strict mode enabled (`"strict": true` in tsconfig.json)
- `allowJs` and `checkJs` both true
- `moduleResolution: "bundler"`

**Rust:** Standard Rust type system, no unsafe code observed

## Error Handling

**Rust (Tauri):**
`.expect()` on builder run — panics on startup failure
```rust
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

**TypeScript:**
No error handling patterns established yet (scaffold only)

## Comments

**Style:** Explanatory comments for non-obvious decisions only
- Rust: `// Prevents additional console window on Windows in release, DO NOT REMOVE!!`
- TypeScript: SvelteKit/Tauri integration rationale in `+layout.ts`

## Tooling Enforcement

- `svelte-check` for frontend type checking (`npm run check`)
- `cargo clippy -- -D warnings` for Rust linting (warnings treated as errors)
- `cargo fmt` for Rust formatting
- No ESLint or Prettier configured yet for frontend
