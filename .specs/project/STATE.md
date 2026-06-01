# State

## Decisions

| ID | Decision | Context | Date |
|----|----------|---------|------|
| D1 | Hybrid git execution (git2 reads, shell-out writes) | ADR-0001 — compatibility vs. performance trade-off | Pre-existing |
| D2 | Cargo workspace: gitty-core, gitty-cli, gitty-tauri | ADR-0002 — shared logic, independent binaries | Pre-existing |
| D3 | English only for all code and documentation | ADR-0003 | Pre-existing |
| D4 | Cursor-inspired design system from DESIGN.md | Warm cream canvas, editorial typography, hairline depth | Pre-existing |

## Blockers

_None currently._

## Lessons

_None yet — project just initialized._

## Deferred Ideas

| Idea | Reason | Source |
|------|--------|--------|
| Dependency Map | Complexity; core features work without it | CONTEXT.md — explicitly v2 |
| GitHub/GitLab API integration | Out of v1 scope | PROJECT.md |

## Preferences

_None recorded yet._

## Todos

- [ ] Set up Cargo workspace structure (3 crates per ADR-0002)
- [ ] Add `git2` and `clap` dependencies
- [ ] Create initial Config schema
