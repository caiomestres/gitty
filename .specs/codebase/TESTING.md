# Testing Infrastructure

**Analyzed:** 2026-06-06

## Test Frameworks

**Unit/Integration (Backend):** `cargo test` — 191 tests across the workspace (155 in core, 26 in tauri, 10 in CLI)
**Unit (Frontend):** Vitest 3.x (configured in `package.json`, smoke test exists at `src/lib/smoke.test.ts`)
**E2E:** None configured
**Coverage:** None configured

## Test Organization

**Naming:**
- Rust: `#[cfg(test)] mod tests` inline + `tests/` directory for integration tests
- Frontend: `*.test.ts` (Vitest convention) — currently only smoke test

**Structure:**
- `crates/gitty-core/src/*.rs` — inline `mod tests` in each module
- `crates/gitty-core/tests/integration_m5.rs` — cross-module integration tests
- `crates/gitty-core/tests/git_read.rs` — git2 read integration tests
- `crates/gitty-core/tests/scan.rs` — scan integration tests
- `src-tauri/src/**/*.rs` — inline `mod tests` in state, error, and command modules
- `src/lib/smoke.test.ts` — frontend smoke test

## Testing Patterns

### Unit Tests (Rust)

**Pattern:** Each module has an inline `#[cfg(test)] mod tests` block with helper factory functions.

**Key patterns:**
- Factory functions for test data: `fresh_status()`, `stale_status(days)`, `make_repo(state)`, `config_simple(interval, last_run)`
- `tempfile::tempdir()` for filesystem-dependent tests (config, health cache, PID files)
- `git2::Repository::init()` for tests requiring real Git repos (changes, health)
- Deterministic time via `OffsetDateTime::now_utc()` parameter (health checks accept `now`)
- Serde round-trip tests for all serializable types

**Coverage by module:**
| Module | Tests | Key scenarios |
|--------|-------|---------------|
| health.rs | 18 | Each check (stale/diverged/dirty/detached) at all severity levels, workspace score, missing repos, empty workspace |
| health_cache.rs | 6 | Save/load round-trip, missing file, corrupt file, directory creation, overwrite, timestamp validation |
| scheduler/mod.rs | 14 | should_run (first run, after interval, before interval, disabled, battery), advanced triggers (window, day, midnight crossing), record_run, compute_next_run, serde round-trip |
| scheduler/daemon.rs | 5 | PID file write/read, cleanup, status without PID, stale PID detection, stop without PID |
| notification.rs | 9 | Each trigger mode (disabled, on-critical, on-any-change, on-scheduler-complete), purge expired, singular/plural messages |
| changes.rs | 8 | Scan with commits, empty repo, missing repo, grouping (author/repo/branch), field population, time window cutoffs |
| power.rs | 1 | battery_state returns valid values (smoke test) |
| config/mod.rs | 4 | Load missing, save/load round-trip, unknown schema version, corrupt JSON |
| repository.rs | 3 | Scan root dedup, find by path/id, state serialization |
| macro_def.rs | 4 | Define/list, duplicate name rejection, find by name/id, delete |
| error.rs (tauri) | 15 | CoreError mapping to AppError codes for each variant, hint generation, transient classification |
| state.rs (tauri) | 3 | New state holds config, reload from disk, with_config_write persists |
| commands/groups.rs | 2 | GroupDto includes repo_count, tree node includes repos and children |
| commands/macros.rs | 4 | Step roundtrip (git op, shell), checkout requires branch, macro variables |
| commands/tags.rs | 1 | TagDto includes repo_count |

### Integration Tests (Rust)

**Location:** `crates/gitty-core/tests/integration_m5.rs`
**Tests:** 12
**Pattern:** Cross-module scenarios that exercise multiple core subsystems together.

| Test | Subsystems exercised |
|------|---------------------|
| scheduler_triggers_health_evaluation_and_notification | scheduler + health + notification + git2 |
| corrupt_health_cache_triggers_fresh_evaluation | health_cache + health |
| stale_pid_file_allows_scheduler_start | daemon (PID management) |
| battery_below_threshold_pauses_scheduler | scheduler + power |
| empty_repo_health_checks_skip_appropriately | health + git2 |
| advanced_trigger_respects_day_constraint | scheduler (advanced mode) |
| advanced_trigger_midnight_crossing_window | scheduler (window logic) |
| compute_next_run_advanced_returns_valid_slot | scheduler (next-run computation) |
| notification_purge_respects_ttl | notification (TTL purge) |
| scheduler_config_serde_round_trip | scheduler (serialization) |
| battery_state_returns_valid_values | power (smoke test) |
| display_name_returns_directory_name | repository (display_name method) |

### E2E Tests

Not configured. Tauri 2 supports WebDriver-based testing but nothing is set up.

## Test Execution

**Commands:**
- Quick (unit only): `cargo test -p gitty-core`
- Full (all crates): `cargo test`
- With linting: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- Type checking: `npm run check` (`svelte-check`)
- Frontend tests: `npm run test:run` (Vitest)
- All checks: `task` (Taskfile — runs frontend + backend)

## Coverage Targets

**Current:** 191 tests total (155 in core, 26 in tauri, 10 in CLI)
**Goals:** Not yet defined
**Enforcement:** GitHub Actions CI runs all tests on every push/PR

## Test Coverage Matrix

| Code Layer | Required Test Type | Location Pattern | Run Command |
|---|---|---|---|
| gitty-core (Rust) | unit + integration | `crates/gitty-core/src/**` + `crates/gitty-core/tests/` | `cargo test -p gitty-core` |
| gitty-cli (Rust) | integration | `crates/gitty-cli/tests/` | `cargo test -p gitty-cli` |
| gitty-tauri (Rust) | unit (command logic) | `src-tauri/src/**` | `cargo test -p gitty-tauri` |
| Svelte components | unit | `src/**/*.test.ts` | `npm run test:run` |
| IPC integration | integration | TBD | TBD |

## Parallelism Assessment

| Test Type | Parallel-Safe? | Isolation Model | Evidence |
|---|---|---|---|
| Rust unit | Yes | No shared mutable state | Pure functions in gitty-core |
| Rust integration | Yes | tempfile::tempdir() per test | Each test creates its own temp dir |
| Svelte unit | Yes | Component isolation | No shared state expected |

## Gate Check Commands

| Gate Level | When to Use | Command |
|---|---|---|
| Quick | After tasks with unit tests only | `cargo test -p gitty-core` |
| Full | After tasks with integration tests | `cargo test && npm run check` |
| Build | After phase completion | `task` (runs all frontend + backend checks) |
