# State

## Decisions

| ID | Decision | Context | Date |
|----|----------|---------|------|
| D1 | Hybrid git execution (git2 reads, shell-out writes) | ADR-0001 — compatibility vs. performance trade-off | Pre-existing |
| D2 | Cargo workspace: gitty-core, gitty-cli, gitty-tauri | ADR-0002 — shared logic, independent binaries | Pre-existing |
| D3 | English only for all code and documentation | ADR-0003 | Pre-existing |
| D4 | Cursor-inspired design system from DESIGN.md | Warm cream canvas, editorial typography, hairline depth | Pre-existing |
| D5 | Foundation (Milestone 1) decomposed into 3 vertical-slice features: `foundation-discovery`, `foundation-git-write`, `foundation-lock` | Milestone too large to build/verify in one pass | 2026-06-01 |
| D6 | Config = single JSON file with schema `version`, no migrations during 0.x, hard-error on mismatch | ADR-0004 | 2026-06-01 |
| D7 | Repository identity = root-commit fingerprint; collision-safe auto re-link; `missing` state for vanished paths | ADR-0005 | 2026-06-01 |
| D8 | Scan = walkdir, descend into nested repos, never into `.git`, default ignore list, no symlinks, non-bare repos only | Grilling — balances nested discovery vs. <5s scan goal | 2026-06-01 |
| D9 | git2 (vendored libgit2) for reads; status exposes branch/detached/dirty/ahead-behind/upstream + HEAD commit summary + changed files | ADR-0001; E2 expansion for dashboard | 2026-06-01 |
| D10 | Git write = shell-out runner `(path, args) -> {code, stdout, stderr}`; git located via PATH, validated at startup (`foundation-git-write` feature) | ADR-0001 | 2026-06-01 |
| D11 | Lock = per-repo PID+timestamp lock file in config dir, stale detection, fail-fast on contention (`foundation-lock` feature) | ADR-0006 | 2026-06-01 |
| D12 | Errors = `thiserror` in gitty-core, `anyhow` at CLI boundary | Idiomatic Rust, tiny deps | 2026-06-01 |

## Blockers

_None currently._

## Lessons

- git2 0.21 accessors return `Result` (not `Option`) for `Reference::shorthand`/`name`, `Commit::summary`, `Buf::as_str`. Check the registry source when unsure.
- `git2` with `default-features = false` builds vendored libgit2 on Windows in ~46s with no cmake/OpenSSL — reads are local-only, so the `ssh`/`https` features are unnecessary.
- The CLI binary is named `gitty` via an explicit `[[bin]]` in `gitty-cli` (package name would otherwise be `gitty-cli`).

## Deferred Ideas

| Idea | Reason | Source |
|------|--------|--------|
| Dependency Map | Complexity; core features work without it | CONTEXT.md — explicitly v2 |
| GitHub/GitLab API integration | Out of v1 scope | PROJECT.md |
| Manual re-link resolution UI (for ambiguous fingerprint matches) | `foundation-discovery` auto-links only unambiguous matches | ADR-0005 |
| Submodule-aware discovery (flag/filter submodules) | `foundation-discovery` treats nested repos generically | Grilling D1 |
| Time-windowed Change Dashboard (24h/7d/30d, group by author/repo/branch) | Aggregate history — Milestone 5 | ROADMAP.md |

## Preferences

_None recorded yet._

## Todos

- [x] Set up Cargo workspace structure (3 crates per ADR-0002)
- [x] Add `git2`, `dirs`, `walkdir`, `thiserror` deps to gitty-core; `anyhow` to gitty-cli (+ `time`, `dunce`)
- [x] Verify `git2` (vendored libgit2) builds on Windows before committing
- [x] `foundation-discovery` feature — config, repository, scan, git::read, re-link, CLI (scan/list/status) — all 12 DISC reqs verified
- [ ] `foundation-git-write` feature — git write layer (shell-out runner, fetch/pull)
- [ ] `foundation-lock` feature — lock mechanism (per-repo PID lock files)
