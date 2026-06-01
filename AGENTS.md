# Gitty

Desktop Git client built with Tauri 2 + SvelteKit 5 + Rust. Includes a standalone Rust CLI.

## Tech Stack

| Layer       | Tech                 | Notes                           |
|-------------|----------------------|---------------------------------|
| Desktop     | Tauri 2              | IPC, native APIs, security      |
| Frontend    | SvelteKit 5 + TS     | Static adapter, Vite            |
| Backend     | Rust                 | Tauri commands + standalone CLI  |
| Tests (FE)  | Vitest               | Unit + integration              |
| Tests (BE)  | cargo test           | Unit + integration              |
| Lint (FE)   | svelte-check         | `npm run check`                 |
| Lint (BE)   | clippy               | `cargo clippy -- -D warnings`   |

## Project Structure

```
gitty/
├── src/                  # SvelteKit frontend
│   └── routes/           # Pages and layouts
├── src-tauri/            # Rust backend
│   ├── src/              # Tauri app + CLI code
│   ├── capabilities/     # Tauri permission definitions
│   └── Cargo.toml
├── static/               # Static assets
├── docs/
│   ├── agents/           # Agent skill config (issue tracker, triage, domain)
│   └── adr/              # Architecture Decision Records (created lazily)
├── .agents/skills/       # LLM-agnostic agent skills
├── DESIGN.md             # UI design system (MUST READ before UI work)
├── AGENTS.md             # This file
└── package.json
```

## Mandatory Rules

### UI Changes

**Before ANY frontend or UI change, reference `DESIGN.md` first.** It contains the complete design system: colors, typography, spacing, components, and do's/don'ts. Every UI decision must align with it. No exceptions.

Key constraints from the design system:
- Warm cream canvas (`#f7f7f4`), never pure white
- Cursor Orange (`#f54e00`) only for primary CTAs — used scarcely
- Display weight stays at 400 — magazine voice, never bold
- Hairline-only depth — no drop shadows
- Inter as CursorGothic substitute, JetBrains Mono for code surfaces

### Security (Tauri)

Follow the `tauri` skill for all Tauri configuration, IPC commands, and capability changes. Key rules:
- Least privilege for capabilities
- Validate all IPC inputs
- CSP configured restrictively
- Never expose `TAURI_` env vars to frontend

### Code Style

- Rust: follow `cargo clippy` and `cargo fmt`
- Svelte/TS: follow `svelte-check` and project ESLint config
- Prefer async Tauri commands over blocking ones
- Tests before implementation when complexity warrants it (see `tdd` skill)

## Development Commands

```bash
# Frontend dev server
npm run dev

# Tauri dev (frontend + backend)
npm run tauri dev

# Build
npm run tauri build

# Type checking
npm run check

# Rust tests
cd src-tauri && cargo test

# Rust linting
cd src-tauri && cargo clippy -- -D warnings

# Rust formatting
cd src-tauri && cargo fmt
```

## Skills Catalog

Skills live in `.agents/skills/`. Invoke by name or trigger phrase.

### Planning & Specification

| Skill | When to Use |
|-------|-------------|
| `grill-with-docs` | Stress-test a plan against the domain model. Extracts context through relentless questioning. Updates CONTEXT.md and ADRs inline. **Use before starting any significant feature.** |
| `tlc-spec-driven` | Full feature lifecycle: Specify → Design → Tasks → Execute. Auto-sizes depth by complexity. Use for features, not quick fixes. |
| `technical-design-doc-creator` | Create TDDs/RFCs for architectural decisions. Use before complex implementations. |
| `to-prd` | Synthesize current conversation into a PRD. Use after a grilling session to formalize requirements. |
| `to-issues` | Break a plan into vertical-slice issues. Use after spec/PRD is ready. |

### Development

| Skill | When to Use |
|-------|-------------|
| `tdd` | Test-driven development: red → green → refactor. Vertical slices, not horizontal. |
| `tauri` | Tauri framework patterns, security, IPC, capabilities. HIGH-RISK skill — always follow for Tauri code. |
| `frontend-design` | Create distinctive, polished UI. Consult `DESIGN.md` first, then use this skill for implementation quality. |
| `diagnose` | Disciplined bug diagnosis: reproduce → minimise → hypothesise → instrument → fix. Use for hard bugs. |
| `playwright-skill` | Browser automation and end-to-end testing of the webview UI. Take screenshots, drive flows, validate responsive behaviour. |

### Frontend Quality

Apply to the SvelteKit webview only. Gitty is a desktop app, not a website — these are about in-app UI quality, not SEO or public-web concerns. **Always defer to `DESIGN.md` and `frontend-design` first.**

| Skill | When to Use |
|-------|-------------|
| `web-design-guidelines` | Review UI code against interface/UX best practices. Use to audit existing UI, not to design it from scratch. |
| `accessibility` | WCAG 2.1 audit: keyboard nav, focus order, contrast, screen-reader support for the webview. |
| `core-web-vitals` | Diagnose perceived-perf issues in the webview (LCP/INP/CLS analogues). Layout shift, slow interactions. |
| `perf-web-optimization` | Reduce webview bundle size, lazy-load heavy components, optimize asset loading. |

### Architecture

| Skill | When to Use |
|-------|-------------|
| `improve-codebase-architecture` | Find deepening opportunities. Surfaces shallow modules, suggests refactors. Produces HTML report. |
| `modular-decomposition` | Analyze coupling, find duplication, group components into domain-aligned **modules**. Structural/boundary level. |
| `tactical-ddd` | Code-level domain modeling inside `src-tauri` (rich entities, value objects, aggregates). Detect/fix anemic models. Distinct from `modular-decomposition` (module boundaries) and `grill-with-docs` (glossary/`CONTEXT.md`). |
| `zoom-out` | Fast, one-shot higher-level perspective of an unfamiliar code area. |
| `codenavi` | Deep investigation of unfamiliar territory with a persistent `.notebook/` knowledge base that grows across sessions. Use when `zoom-out` isn't enough and you need durable navigation notes. |
| `mermaid-studio` | Generate/validate/render Mermaid diagrams (architecture, sequence, ERD, C4) to visualize systems and flows. |

### Security

Precedence when guidance conflicts: **`tauri` (authoritative for IPC/capabilities/CSP) → `security-best-practices` (Rust/TS language-level) → `security-threat-model` (whole-feature threat enumeration) → `best-practices` (generic web hygiene, lowest priority).**

| Skill | When to Use |
|-------|-------------|
| `security-best-practices` | Language/framework-specific secure-coding review (Rust backend, TS frontend). |
| `security-threat-model` | Enumerate trust boundaries, assets, abuse paths, and mitigations before a risky feature. |
| `best-practices` | Generic web/code hygiene. Lowest priority — defer to the skills above on any overlap. |

### Workflow & Agents

| Skill | When to Use |
|-------|-------------|
| `triage` | Triage issues through a state machine. Use for issue management. |
| `handoff` | Compact conversation context for another agent to continue. Use when switching sessions. |
| `caveman` | Ultra-compressed communication. ~75% token savings. Say "caveman mode" to activate. |
| `gh-fix-ci` | Debug and fix failing GitHub Actions PR checks. Inspects checks/logs via `gh`, drafts a fix plan, implements after approval. |
| `setup-matt-pocock-skills` | Configure issue tracker, triage labels, domain docs. Run once to enable `triage`, `to-issues`, `to-prd`. |

### Meta / Skill Creation

| Skill | When to Use |
|-------|-------------|
| `skill-architect` | Design new skills through structured conversation. |
| `write-a-skill` | Create skill files with proper structure. |
| `subagent-creator` | Create specialized subagents for isolated workflows. |

## Skill Chaining Patterns

Common workflows that chain multiple skills:

### Primary Workflow: Feature Development

```
grill-with-docs → to-prd → tlc-spec-driven (Specify → Design → Tasks) → to-issues → Execute → close-out (update PRD + close issues)
```

This is an OSS project — work is tracked in the open. `to-prd` and `to-issues` are **mandatory**, not optional.

**Hand-off points:**

1. **`grill-with-docs` runs FIRST** for any significant feature. Produces:
   - `CONTEXT.md` — sharpened domain language (used by all downstream skills)
   - `docs/adr/` — hard-to-reverse decisions (constraints for design phase)
   - Validated, stress-tested understanding of scope

2. **`to-prd` is MANDATORY.** After grilling, publish or refresh the PRD on the issue tracker before speccing.
   - If no PRD issue exists for the scope, create one.
   - If a PRD issue already exists (e.g. the v1 PRD, issue #1), do NOT create a duplicate — extend it and track the feature there (see close-out).
   - The PRD is NOT a replacement for `spec.md` — it's higher-level documentation for stakeholders.

3. **`tlc-spec-driven` picks up after the PRD.** Entry point: `specify feature`. It:
   - Reads `CONTEXT.md` for vocabulary (MUST use canonical terms)
   - Reads `docs/adr/` for architectural constraints
   - Skips redundant clarification — the grilling already happened
   - Proceeds through Specify → Design → Tasks (auto-sized by complexity)

4. **`to-issues` is MANDATORY** after the Tasks phase. Break the feature into vertical-slice issues, quiz the user on granularity, then publish each as an issue linked to the PRD (as Parent) with the `ready-for-agent` label. Execute the work against these issues.

5. **Close-out is MANDATORY when a feature completes.** Keep the tracker honest:
   - Comment progress on the PRD issue and tick the feature's box in the PRD's Progress checklist (add one if absent).
   - Close the feature's issue(s) with a comment referencing the feature `spec.md` and the delivering commit/PR (`Closes #N`).
   - Close the PRD issue ONLY when every feature in its scope has shipped — otherwise it stays open.

### Secondary Workflows

| Workflow | Chain | When |
|----------|-------|------|
| Bug Fix | `diagnose` → fix → `improve-codebase-architecture` (if structural) | Hard bugs |
| Quick Fix | `tlc-spec-driven` quick mode | ≤3 files, obvious fix |
| Architecture Review | `zoom-out` → `improve-codebase-architecture` → `grill-with-docs` | Before refactors |
| Session Handoff | finish work → `handoff` → new agent picks up | Switching sessions |
| Issue Triage | `triage` | Incoming bugs/requests |

**If Apply** Suggest a git message for that workflow at the end.

### Skills That Stay Independent

These skills are NOT part of the primary chain — they're invoked based on situation:

- `tdd` — can be used during tlc-spec-driven's Execute phase for test-first implementation
- `diagnose` — independent workflow triggered by bugs
- `triage` — independent workflow for issue lifecycle
- `tauri` — always consulted for Tauri-specific code (security, IPC, capabilities)
- `frontend-design` — always consulted alongside `DESIGN.md` for UI work

## Domain Documentation

### CONTEXT.md

When created, lives at project root. Contains domain glossary only — no implementation details. Skills that read it: `grill-with-docs`, `improve-codebase-architecture`, `diagnose`, `tdd`, `triage`.

### ADRs

When created, live in `docs/adr/`. Record hard-to-reverse architectural decisions. Format defined in `.agents/skills/grill-with-docs/ADR-FORMAT.md`.

## Agent Skills

### Issue tracker

GitHub Issues via `gh` CLI. See `docs/agents/issue-tracker.md`.

**Hard rule (applies to the mandatory `to-prd`/`to-issues`/close-out steps):** write every issue/PRD body and comment from a UTF-8 file via `--body-file` — never inline `--body "..."`/heredoc, and never pipe `gh` output through PowerShell `Out-File`/`>` (it double-encodes UTF-8 and corrupts the body). Full rationale and verification steps in `docs/agents/issue-tracker.md`.

### Triage labels

Default canonical names (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout. `CONTEXT.md` + `docs/adr/` at repo root. See `docs/agents/domain.md`.
