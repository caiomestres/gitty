# M4 GUI Tasks

**Design**: `.specs/features/m4-gui/design.md`
**Status**: Draft

---

## Execution Plan

### Phase 1: Infrastructure (Sequential)

Refactor the Tauri layer before adding new commands.

```
T1 → T2 → T3 → T4
```

### Phase 2: IPC Commands (Parallel OK)

New Tauri commands — each module is independent.

```
      ┌→ T5 (Group commands) ─┐
T4 ──┼→ T6 (Tag commands)    ├──→ T9
      └→ T7 (Macro commands)  ┘
      T8 (Sidebar nav fix) ──→
```

### Phase 3: Frontend — Core Pages (Parallel OK)

UI components that depend on IPC commands being available.

```
       ┌→ T9  (Sidebar tree)     ─┐
T5-T8 ┼→ T10 (Groups page)       ├──→ T13
       ├→ T11 (Repo detail enh.)  │
       └→ T12 (Dashboard filter)  ┘
```

### Phase 4: Frontend — Macros (Sequential)

Complex Macro builder depends on Macro IPC.

```
T7 → T13 (Macro list + builder) → T14 (Macro execution + results)
```

---

## Task Breakdown

### T1: Split src-tauri/src/lib.rs into modules

**What**: Refactor the existing monolith into `state.rs`, `error.rs`, `commands/workspace.rs` modules. No behavior change — pure refactor.
**Where**: `src-tauri/src/lib.rs` → `src-tauri/src/state.rs`, `src-tauri/src/error.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/workspace.rs`
**Depends on**: None
**Reuses**: All existing code, just reorganized
**Requirement**: M4GUI-01, M4GUI-02

**Done when**:
- [ ] `lib.rs` delegates to modules (< 50 lines)
- [ ] `commands/workspace.rs` contains the 10 existing commands
- [ ] `state.rs` has `AppState` struct with `Mutex<Config>` + config path
- [ ] `error.rs` has `AppError` struct with `code` + `message` fields, `From<CoreError>` impl
- [ ] All 10 existing commands work identically (behavioral equivalence)
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 70+ tests pass (no regressions)

**Tests**: unit (AppError mapping tests)
**Gate**: full
**Commit**: `refactor(tauri): split lib.rs into state, error, and command modules`

---

### T2: Implement managed state with `notify` file-watcher

**What**: Add `notify` dependency, implement file-watcher on config path, wire into `AppState`, emit Tauri event on config change.
**Where**: `src-tauri/src/state.rs`, `src-tauri/Cargo.toml`
**Depends on**: T1
**Reuses**: `Config::load()`, `Config::config_path()`
**Requirement**: M4GUI-01

**Done when**:
- [ ] `notify` added to `src-tauri/Cargo.toml`
- [ ] `AppState::start_watcher()` spawns a file-watcher thread
- [ ] On config file modification, `AppState::reload()` is called
- [ ] `app_handle.emit("config-changed", ())` fires on reload
- [ ] `with_config_write` saves to disk AND updates in-memory state
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 72+ tests pass

**Tests**: unit (watcher start/reload logic)
**Gate**: full
**Commit**: `feat(tauri): add managed state with notify file-watcher (ADR-0007)`

---

### T3: Migrate existing commands to structured errors

**What**: Change all 10 existing commands from `Result<T, String>` to `Result<T, AppError>`. Implement `From<CoreError>` and `From<String>` for `AppError`. Ensure Tauri serializes `AppError` correctly.
**Where**: `src-tauri/src/error.rs`, `src-tauri/src/commands/workspace.rs`
**Depends on**: T1
**Reuses**: Existing command implementations
**Requirement**: M4GUI-02

**Done when**:
- [ ] All 10 commands return `Result<T, AppError>` instead of `Result<T, String>`
- [ ] `AppError` implements `Serialize` and Tauri's error trait
- [ ] Error codes properly mapped from `CoreError` variants
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 72+ tests pass

**Tests**: unit (error code mapping tests)
**Gate**: full
**Commit**: `refactor(tauri): migrate all commands to structured AppError`

---

### T4: Migrate existing commands to managed state

**What**: Change all 10 existing commands to use `State<'_, AppState>` instead of loading config from disk on each call.
**Where**: `src-tauri/src/commands/workspace.rs`, `src-tauri/src/lib.rs` (managed state registration)
**Depends on**: T1, T2, T3
**Reuses**: `AppState` from T1/T2
**Requirement**: M4GUI-01

**Done when**:
- [ ] All 10 commands take `state: State<'_, AppState>` parameter
- [ ] `with_config_read` removed (replaced by `state.config()`)
- [ ] `with_config_write` removed (replaced by `state.with_config_write()`)
- [ ] `AppState` registered via `tauri::Builder::manage()`
- [ ] File-watcher started in `setup()` hook
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 72+ tests pass

**Tests**: unit
**Gate**: full
**Commit**: `refactor(tauri): migrate all commands to managed state`

---

### T5: Add Group IPC commands [P]

**What**: Implement 7 Group commands: `list_groups`, `create_group`, `rename_group`, `delete_group`, `move_group`, `assign_repo_to_group`, `group_tree`.
**Where**: `src-tauri/src/commands/groups.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
**Depends on**: T4
**Reuses**: `gitty_core::group::*`, `AppState`, `AppError`
**Requirement**: M4GUI-03

**Done when**:
- [ ] 7 commands implemented with correct DTOs (`GroupDto`, `GroupTreeNodeDto`)
- [ ] Commands registered in `generate_handler![]`
- [ ] `GroupTreeNodeDto` includes `repos: Vec<RepoDto>` for sidebar consumption
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 74+ tests pass

**Tests**: unit (DTO mapping, at least 2 command tests)
**Gate**: full
**Commit**: `feat(tauri): add 7 Group IPC commands`

---

### T6: Add Tag IPC commands [P]

**What**: Implement 3 Tag commands: `list_tags`, `add_tag`, `remove_tag`.
**Where**: `src-tauri/src/commands/tags.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
**Depends on**: T4
**Reuses**: `gitty_core::tag::*`, `AppState`, `AppError`
**Requirement**: M4GUI-04

**Done when**:
- [ ] 3 commands implemented with `TagDto`
- [ ] Commands registered in `generate_handler![]`
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 73+ tests pass

**Tests**: unit (DTO mapping)
**Gate**: full
**Commit**: `feat(tauri): add 3 Tag IPC commands`

---

### T7: Add Macro IPC commands [P]

**What**: Implement 5 Macro commands: `list_macros`, `get_macro`, `define_macro`, `delete_macro`, `run_macro`.
**Where**: `src-tauri/src/commands/macros.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
**Depends on**: T4
**Reuses**: `gitty_core::macro_def::*`, `gitty_core::execution::*`, `gitty_core::selection::*`, `AppState`, `AppError`
**Requirement**: M4GUI-05

**Done when**:
- [ ] 5 commands implemented with DTOs (`MacroDto`, `StepDto`, `StepKindDto`, `SelectionDto`, `JobDto`, `StepResultDto`)
- [ ] `run_macro` resolves selection, calls `execute_macro`, returns `Vec<JobDto>`
- [ ] `define_macro` converts `StepDto` → core `Step` types
- [ ] Commands registered in `generate_handler![]`
- [ ] Gate: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] Test count: 75+ tests pass

**Tests**: unit (DTO conversion, at least 2 command tests)
**Gate**: full
**Commit**: `feat(tauri): add 5 Macro IPC commands`

---

### T8: Update sidebar navigation [P]

**What**: Fix dead links, update nav items to Dashboard, Groups, Macros, Settings.
**Where**: `src/lib/components/Sidebar.svelte`
**Depends on**: None (can be done anytime)
**Reuses**: Existing Sidebar.svelte structure
**Requirement**: M4GUI-14

**Done when**:
- [ ] Nav items: Dashboard (`/`), Groups (`/groups`), Macros (`/macros`), Settings (`/settings`)
- [ ] `/repos` link removed
- [ ] `npm run check` passes

**Tests**: none (trivial nav change)
**Gate**: `npm run check`
**Commit**: `fix(ui): update sidebar nav — remove dead links, add Groups + Macros`

---

### T9: Build sidebar Group tree component

**What**: Rewrite `Sidebar.svelte` to show a collapsible Group tree with Repositories under each Group, plus the fixed nav items at the top.
**Where**: `src/lib/components/Sidebar.svelte`
**Depends on**: T5 (Group IPC), T8 (nav fix)
**Reuses**: Existing sidebar brand styling, CSS tokens
**Requirement**: M4GUI-06

**Done when**:
- [ ] Sidebar calls `invoke("group_tree")` on mount and on `config-changed` event
- [ ] Groups render as collapsible tree nodes (click to toggle)
- [ ] Repositories listed under their Group (clicking navigates to `/repo/[id]`)
- [ ] Missing Repositories shown with muted/dimmed style
- [ ] Ungrouped Group shown at top level
- [ ] Fixed nav items (Dashboard, Groups, Macros, Settings) above the tree
- [ ] `npm run check` passes

**Tests**: none (visual component — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): sidebar Group tree with collapsible nodes`

---

### T10: Build Groups admin page [P]

**What**: Create `/groups` route with Group list, Create/Rename/Delete/Move actions.
**Where**: `src/routes/groups/+page.svelte`
**Depends on**: T5 (Group IPC)
**Reuses**: Dialog pattern from dashboard scan dialog, button styles
**Requirement**: M4GUI-07

**Done when**:
- [ ] Groups page shows flat list: name, parent name, repo count
- [ ] "Create Group" button opens dialog (name + optional parent dropdown)
- [ ] Rename: inline edit or dialog
- [ ] Delete: confirmation dialog, shows warning about repo reassignment
- [ ] Move: parent selector dropdown
- [ ] Error messages displayed for duplicate names, delete Ungrouped, cycle detection
- [ ] `npm run check` passes

**Tests**: none (visual page — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): Groups admin page with CRUD operations`

---

### T11: Enhance Repository detail — Group dropdown + Tag editor [P]

**What**: Add Group assignment dropdown and inline Tag editor to the Repository detail page.
**Where**: `src/routes/repo/[id]/+page.svelte`, `src/lib/types/workspace.ts`
**Depends on**: T5 (Group IPC), T6 (Tag IPC)
**Reuses**: Existing detail page layout, info-card pattern
**Requirement**: M4GUI-08, M4GUI-09

**Done when**:
- [ ] Group section shows current Group name in a `<select>` dropdown
- [ ] Changing the dropdown shows "Moving from [Old] to [New]" and calls `assign_repo_to_group` on confirm
- [ ] Tag section shows current Tags as removable pills + "Add Tag" input
- [ ] Adding a Tag calls `add_tag` and updates the list
- [ ] Removing a Tag calls `remove_tag` and removes the pill
- [ ] Empty tag input shows validation error
- [ ] New DTO types added to `workspace.ts`
- [ ] `npm run check` passes

**Tests**: none (visual — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): repository detail Group dropdown and Tag editor`

---

### T12: Add Dashboard Tag filter [P]

**What**: Add Tag filter dropdown above the repo table on the dashboard.
**Where**: `src/routes/+page.svelte`
**Depends on**: T6 (Tag IPC)
**Reuses**: Existing dashboard layout, stats bar pattern
**Requirement**: M4GUI-10

**Done when**:
- [ ] Dashboard calls `invoke("list_tags")` on mount
- [ ] A dropdown appears above the repo table with all Tags + "All" option
- [ ] Selecting a Tag filters the displayed repos (client-side)
- [ ] Clearing the filter shows all repos
- [ ] Empty state message when no repos match
- [ ] `npm run check` passes

**Tests**: none (visual — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): dashboard Tag filter dropdown`

---

### T13: Build Macro list + builder page

**What**: Create `/macros` route with Macro list and full visual step builder.
**Where**: `src/routes/macros/+page.svelte`, `src/lib/types/workspace.ts`
**Depends on**: T7 (Macro IPC)
**Reuses**: Dialog pattern, form patterns
**Requirement**: M4GUI-11

**Done when**:
- [ ] Macros page shows list of all Macros (name, step count)
- [ ] "New Macro" opens builder form with name input + step list
- [ ] Each step has type selector (Git Op / Shell)
- [ ] Git Op steps: dropdown (Fetch/Pull/Checkout) + branch input for Checkout
- [ ] Shell steps: command text input + optional label
- [ ] Each step: optional condition input, optional confirm toggle
- [ ] Each step: optional rollback (nested step editor, one level)
- [ ] Step reorder via move up/down buttons
- [ ] Variable editor: key-value pairs with add/remove
- [ ] Save calls `define_macro`, updates list
- [ ] Edit loads existing Macro into builder (delete + redefine on save)
- [ ] Delete with confirmation dialog
- [ ] `npm run check` passes

**Tests**: none (complex visual form — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): Macro list and visual step builder`

---

### T14: Build Macro execution + Job results panel

**What**: Add Selection dialog and inline Job results panel to the Macros page.
**Where**: `src/routes/macros/+page.svelte` (extend)
**Depends on**: T13 (Macro page exists), T5 (Group list for selector), T6 (Tag list for selector)
**Reuses**: Dialog pattern, badge styles
**Requirement**: M4GUI-12, M4GUI-13

**Done when**:
- [ ] "Run" button on each Macro opens Selection dialog
- [ ] Selection dialog: radio (All / Group / Tag / Individual)
- [ ] Group selection: dropdown populated from `list_groups`
- [ ] Tag selection: dropdown populated from `list_tags`
- [ ] Individual: checkbox list of all active Repositories
- [ ] "Execute" calls `run_macro` with appropriate `SelectionDto`
- [ ] Results panel appears inline showing per-Repository status
- [ ] Success: green indicator + repo name
- [ ] Failed: red indicator + error message
- [ ] Skipped: muted indicator + reason
- [ ] Summary line: "N succeeded, N failed, N skipped"
- [ ] Dismiss button closes the results panel
- [ ] `npm run check` passes

**Tests**: none (visual — manual verification)
**Gate**: `npm run check`
**Commit**: `feat(ui): Macro execution with Selection dialog and Job results`

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 → T2
  T1 → T3
  T2 + T3 → T4

Phase 2 (Parallel):
  T4 complete, then:
    ├── T5 [P] (Group IPC)
    ├── T6 [P] (Tag IPC)
    ├── T7 [P] (Macro IPC)
    └── T8 [P] (Sidebar nav fix)

Phase 3 (Parallel, after Phase 2):
  T5 + T8 → T9 (Sidebar tree)
  T5 → T10 [P] (Groups page)
  T5 + T6 → T11 [P] (Repo detail)
  T6 → T12 [P] (Dashboard filter)

Phase 4 (Sequential):
  T7 → T13 (Macro builder)
  T13 + T5 + T6 → T14 (Macro execution)
```

---

## Task Granularity Check

| Task | Scope | Status |
|------|-------|--------|
| T1: Split lib.rs into modules | 1 refactor (file reorg) | ✅ Granular |
| T2: Managed state + watcher | 1 module (state.rs) | ✅ Granular |
| T3: Structured errors | 1 module (error.rs) + migration | ✅ Granular |
| T4: Migrate commands to state | 1 file (workspace.rs) | ✅ Granular |
| T5: Group IPC (7 commands) | 1 module (groups.rs) | ⚠️ 7 commands but all trivial delegation |
| T6: Tag IPC (3 commands) | 1 module (tags.rs) | ✅ Granular |
| T7: Macro IPC (5 commands) | 1 module (macros.rs) | ⚠️ 5 commands but cohesive |
| T8: Sidebar nav fix | 1 component tweak | ✅ Granular |
| T9: Sidebar tree | 1 component rewrite | ✅ Granular |
| T10: Groups page | 1 route page | ✅ Granular |
| T11: Repo detail enhancements | 1 page modification | ✅ Granular |
| T12: Dashboard filter | 1 page modification | ✅ Granular |
| T13: Macro builder | 1 route page (complex) | ⚠️ Complex but cohesive form |
| T14: Macro execution | 1 page extension | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (body) | Diagram Shows | Status |
|------|-------------------|---------------|--------|
| T1 | None | Start node | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T1 | T1 → T3 | ✅ Match |
| T4 | T1, T2, T3 | T2 + T3 → T4 | ✅ Match |
| T5 | T4 | T4 → T5 [P] | ✅ Match |
| T6 | T4 | T4 → T6 [P] | ✅ Match |
| T7 | T4 | T4 → T7 [P] | ✅ Match |
| T8 | None | Independent | ✅ Match |
| T9 | T5, T8 | T5 + T8 → T9 | ✅ Match |
| T10 | T5 | T5 → T10 [P] | ✅ Match |
| T11 | T5, T6 | T5 + T6 → T11 [P] | ✅ Match |
| T12 | T6 | T6 → T12 [P] | ✅ Match |
| T13 | T7 | T7 → T13 | ✅ Match |
| T14 | T13, T5, T6 | T13 + T5 + T6 → T14 | ✅ Match |
