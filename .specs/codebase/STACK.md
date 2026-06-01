# Tech Stack

**Analyzed:** 2026-05-31

## Core

- Desktop framework: Tauri 2.x
- Frontend framework: SvelteKit 2.9+ / Svelte 5
- Frontend language: TypeScript ~5.6.2
- Backend language: Rust (edition 2021)
- Build tool: Vite 6.x
- Package manager: npm (frontend), Cargo (backend)

## Frontend

- UI Framework: Svelte 5 (runes mode — `$state`, `$derived`)
- Routing: SvelteKit with `adapter-static` (SPA mode, `ssr = false`)
- Styling: Scoped `<style>` blocks (no CSS framework yet)
- State Management: Svelte 5 runes (`$state`)
- IPC: `@tauri-apps/api` v2 (`invoke()`)

## Backend

- Framework: Tauri 2.x (single crate currently, workspace planned per ADR-0002)
- Serialization: `serde` 1.x + `serde_json` 1.x
- Tauri plugins: `tauri-plugin-opener` 2.x
- Git (planned): `git2` for reads, system `git` CLI for writes (ADR-0001)
- CLI (planned): `clap` for argument parsing

## Testing

- Unit (frontend): Vitest (listed in AGENTS.md, not yet configured)
- Unit (backend): `cargo test` (no tests written yet)
- Integration: None configured
- E2E: None configured

## External Services

_None — Gitty is fully local, no cloud dependencies._

## Development Tools

- IDE: Cursor (VS Code base) with `.vscode/settings.json` + `extensions.json`
- Build: `tauri-build` 2.x (Rust build script)
- Type checking: `svelte-check` 4.x
- Linting (Rust): `cargo clippy`
- Formatting (Rust): `cargo fmt`
