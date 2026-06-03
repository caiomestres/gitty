# Tech Stack

**Analyzed:** 2026-06-02

## Core

- Desktop framework: Tauri 2.x
- Frontend framework: SvelteKit 2.9+ / Svelte 5
- Frontend language: TypeScript ~5.6.2
- Backend language: Rust (edition 2021)
- Build tool: Vite 6.x
- Package manager: npm (frontend), Cargo (backend)

## Frontend

- UI Framework: Svelte 5 (runes mode — `$state`, `$derived`, `$effect`)
- Routing: SvelteKit with `adapter-static` (SPA mode, `ssr = false`)
- Styling: Scoped `<style>` blocks + CSS custom properties design system
- State Management: Svelte 5 runes (`$state`, `$derived`)
- IPC: `@tauri-apps/api` v2 (`invoke()`)
- Event listening: `@tauri-apps/api/event` (`listen()` for config-changed events)

## Backend

- Framework: Tauri 2.x (Cargo workspace member: `gitty-tauri`)
- Serialization: `serde` 1.x + `serde_json` 1.x
- Tauri plugins: `tauri-plugin-opener` 2.x
- File watching: `notify` 7.x (config file change detection)
- Git reads: `git2` 0.21 (vendored libgit2, default-features = false)
- Git writes: `std::process::Command` shell-out to system `git` (ADR-0001)
- CLI: `clap` 4.x (derive mode)
- Error handling: `thiserror` 2.x (core), `anyhow` 1.x (CLI boundary)
- Time: `time` 0.3.47 (OffsetDateTime, formatting, serde)
- UUIDs: `uuid` 1.x (v4 generation, serde)
- Battery: `battery` 0.7 (power-state aware scheduling)
- File locking: `fs2` 0.4 (advisory file locks for health cache)
- Filesystem: `walkdir` 2.5, `dirs` 6.0, `dunce` 1.0
- Daemonization: `daemonize` 0.5 (Unix only)

## Testing

- Unit/Integration (backend): `cargo test` — 165 tests total
- Unit (tauri): `cargo test -p gitty-tauri` — 18 tests
- Unit (frontend): Vitest (configured, smoke test exists)
- Integration: `crates/gitty-core/tests/integration_m5.rs`
- E2E: None configured
- Dev dependencies: `tempfile` 3.x, `assert_cmd` 2.x, `predicates` 3.x

## External Services

_None — Gitty is fully local, no cloud dependencies._

## Development Tools

- IDE: Cursor (VS Code base) with `.vscode/settings.json` + `extensions.json`
- Build: `tauri-build` 2.x (Rust build script)
- Type checking: `svelte-check` 4.x
- Linting (Rust): `cargo clippy`
- Formatting (Rust): `cargo fmt`
- Linting (TS): ESLint flat config with `typescript-eslint` + `eslint-plugin-svelte`
- Formatting (TS): Prettier with `prettier-plugin-svelte`
- Git hooks: Husky (pre-commit → lint-staged)
