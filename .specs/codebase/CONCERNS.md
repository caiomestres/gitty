# Concerns

> All concerns from initial brownfield analysis have been resolved.

## Resolved

### ARCH-1: Single crate vs. planned workspace ✅

**Resolution:** Migrated to Cargo workspace. Root `Cargo.toml` defines workspace with members: `crates/gitty-core`, `crates/gitty-cli`, `src-tauri` (gitty-tauri). All three crates compile and test cleanly.

### SEC-1: CSP not configured ✅

**Resolution:** CSP configured in `src-tauri/tauri.conf.json`: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost`

### TEST-1: No tests exist ✅

**Resolution:** Vitest added to `package.json` with smoke test at `src/lib/smoke.test.ts`. Rust scaffold test in `crates/gitty-core/src/lib.rs`. Gate check commands functional: `cargo test`, `npx vitest run`.

### FE-1: Scaffold UI doesn't match design system ✅ (acknowledged)

**Resolution:** Acknowledged — scaffold will be replaced in Milestone 3. No action needed until then.

### FE-2: No ESLint or Prettier configured ✅

**Resolution:** ESLint flat config (`eslint.config.js`) with `typescript-eslint` + `eslint-plugin-svelte` + `svelte-eslint-parser`. Prettier with `prettier-plugin-svelte`. Husky pre-commit hook running `lint-staged`. Scripts: `npm run lint`, `npm run format`, `npm run format:check`.
