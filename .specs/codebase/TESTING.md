# Testing Infrastructure

## Test Frameworks

**Unit/Integration (Frontend):** Vitest (referenced in AGENTS.md, not yet added to `package.json`)
**Unit/Integration (Backend):** `cargo test` (built-in, no tests written yet)
**E2E:** None configured
**Coverage:** None configured

## Test Organization

**Location:** No tests exist yet
**Naming (planned):**
- Frontend: `*.test.ts` or `*.spec.ts` (Vitest convention)
- Rust: `#[cfg(test)] mod tests` inline or `tests/` directory

**Structure:** Not yet established

## Testing Patterns

### Unit Tests

**Frontend:**
Not yet established. Vitest is the intended framework per AGENTS.md.

**Rust:**
Not yet established. Standard `#[test]` functions with `cargo test`.

### Integration Tests

**Frontend:**
Not yet established. Vitest can serve for integration tests with Tauri IPC mocking.

**Rust:**
Not yet established. Planned for `gitty-core` crate (per ADR-0002, core logic is testable without Tauri).

### E2E Tests

Not configured. Tauri 2 supports WebDriver-based testing but nothing is set up.

## Test Execution

**Commands:**
- Frontend: `npx vitest` (once configured)
- Rust: `cd src-tauri && cargo test`
- Type checking: `npm run check` (`svelte-check`)

**Configuration:** No test configuration files exist yet

## Coverage Targets

**Current:** 0% — no tests exist
**Goals:** Not yet defined
**Enforcement:** Not yet automated

## Test Coverage Matrix

| Code Layer | Required Test Type | Location Pattern | Run Command |
|---|---|---|---|
| gitty-core (Rust) | unit + integration | `crates/gitty-core/src/**` + `crates/gitty-core/tests/` | `cargo test -p gitty-core` |
| gitty-cli (Rust) | integration | `crates/gitty-cli/tests/` | `cargo test -p gitty-cli` |
| gitty-tauri (Rust) | unit (command logic) | `crates/gitty-tauri/src/**` | `cargo test -p gitty-tauri` |
| Svelte components | unit | `src/**/*.test.ts` | `npx vitest` |
| IPC integration | integration | TBD | TBD |

> Note: Crate paths assume Cargo workspace migration per ADR-0002. Current single-crate path is `src-tauri/`.

## Parallelism Assessment

| Test Type | Parallel-Safe? | Isolation Model | Evidence |
|---|---|---|---|
| Rust unit | Yes | No shared mutable state | Pure functions in gitty-core |
| Rust integration | Depends | Filesystem-based (Git repos) | Tests touching real repos need temp dirs |
| Svelte unit | Yes | Component isolation | No shared state expected |

## Gate Check Commands

| Gate Level | When to Use | Command |
|---|---|---|
| Quick | After tasks with unit tests only | `cargo test -p gitty-core` |
| Full | After tasks with integration tests | `cargo test && npm run check` |
| Build | After phase completion | `npm run build && cd src-tauri && cargo clippy -- -D warnings && cargo test` |
