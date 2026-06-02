# M4 GUI — Organization & Operations in Desktop App

## Problem Statement

Gitty's M4 features (Groups, Tags, Macros, Macro execution, Job monitoring) exist only in gitty-core and the CLI. The desktop app has zero GUI for any of these. Users who prefer the GUI cannot organize Repositories into Groups, manage Tags, define Macros, or run bulk operations beyond Fetch All / Pull All. The sidebar has dead links to non-existent pages.

## Goals

- [ ] Expose all M4 domain features (Groups, Tags, Macros, Jobs) in the Tauri desktop app
- [ ] Refactor the Tauri IPC layer to support managed state, file-watching, and structured errors
- [ ] Build a VS Code-style sidebar Group tree for Repository navigation
- [ ] Build a visual Macro builder for defining multi-step workflows
- [ ] Enable Macro execution with target selection and inline Job monitoring

## Out of Scope

| Feature | Reason |
|---------|--------|
| Drag-and-drop assignment in sidebar | Complexity; dropdown on detail page is sufficient for v1 |
| Macro JSON import/export | CLI inline syntax is sufficient; file-based deferred |
| Parallel Macro execution (concurrent Jobs) | D15: sequential in v1; parallel requires more Lock work |
| Macro scheduling (background execution) | Milestone 5 scope |
| Health Checks / Workspace Health | Milestone 5 scope |
| Change Dashboard | Milestone 5 scope |

---

## User Stories

### P1: Tauri Infrastructure Refactor ⭐ MVP

**User Story**: As a desktop app user, I want the GUI to respond quickly and reflect CLI changes automatically so that both tools stay in sync.

**Why P1**: Foundation for all M4 GUI commands. Without managed state + file-watcher, every IPC call hits disk and CLI changes are invisible.

**Acceptance Criteria**:

1. WHEN the Tauri app starts THEN it SHALL load Config into managed state (`Mutex<Config>`) and start a file-watcher on the config path
2. WHEN the CLI modifies the config file THEN the file-watcher SHALL reload Config into managed state within 1 second
3. WHEN any Tauri command fails THEN it SHALL return a structured `ErrorDto` with `code` (string enum) and `message` fields
4. WHEN a write command succeeds THEN it SHALL save to disk AND update managed state atomically

**Independent Test**: Start the desktop app, run `gitty group create Test` from CLI, verify the Group appears in the GUI without manual refresh.

---

### P1: Sidebar Group Tree ⭐ MVP

**User Story**: As a desktop app user, I want to see my Groups and Repositories in a collapsible tree in the sidebar so that I can navigate my workspace structure at a glance.

**Why P1**: Primary navigation mechanism. Without it, users have no way to see or use Groups in the GUI.

**Acceptance Criteria**:

1. WHEN the sidebar loads THEN it SHALL display a collapsible tree of all Groups with Repository names nested under their assigned Group
2. WHEN a Group node is collapsed/expanded THEN it SHALL toggle visibility of its children (sub-Groups and Repositories)
3. WHEN a Repository name is clicked THEN the app SHALL navigate to that Repository's detail page
4. WHEN a Repository has no Group assignment THEN it SHALL appear under the "Ungrouped" Group
5. WHEN the config changes (via file-watcher) THEN the sidebar tree SHALL update automatically

**Independent Test**: Create Groups via CLI, assign Repositories, verify the sidebar shows the correct tree structure with collapsible nodes.

---

### P1: Group Admin Page ⭐ MVP

**User Story**: As a desktop app user, I want a page where I can create, rename, delete, and move Groups so that I can organize my workspace hierarchy.

**Why P1**: Users need to manage Groups without the CLI.

**Acceptance Criteria**:

1. WHEN the user navigates to the Groups page THEN it SHALL show a flat list of all Groups with name, parent, and Repository count
2. WHEN the user clicks "Create Group" THEN a dialog SHALL appear with name input and optional parent selector
3. WHEN the user renames a Group THEN the new name SHALL appear in the sidebar tree and Groups page immediately
4. WHEN the user deletes a Group THEN its Repositories SHALL move to Ungrouped and child Groups SHALL be re-parented
5. WHEN the user attempts to delete the "Ungrouped" Group THEN the app SHALL show an error message
6. WHEN a duplicate name exists under the same parent THEN the app SHALL show a validation error

**Independent Test**: Create a nested Group hierarchy, rename a child, delete the parent, verify Repositories move to Ungrouped.

---

### P1: Repository Detail — Group & Tag Management ⭐ MVP

**User Story**: As a desktop app user, I want to assign a Repository to a Group and manage its Tags from the detail page so that I can organize individual Repositories.

**Why P1**: Core organization workflow.

**Acceptance Criteria**:

1. WHEN the detail page loads THEN it SHALL show the current Group name (or "Ungrouped") in a dropdown selector
2. WHEN the user selects a different Group THEN the app SHALL show "Moving from [Old] to [New]" confirmation and update on confirm
3. WHEN the detail page loads THEN it SHALL show an editable Tag list with an "Add Tag" input
4. WHEN the user adds a Tag THEN it SHALL appear immediately in the Tag list
5. WHEN the user removes a Tag THEN it SHALL disappear immediately
6. WHEN the user adds an empty/whitespace Tag THEN the app SHALL show a validation error

**Independent Test**: Open a Repository detail, change its Group, add/remove Tags, verify changes persist after page reload.

---

### P1: Dashboard Tag Filter ⭐ MVP

**User Story**: As a desktop app user, I want to filter the Repository table by Tag so that I can quickly find Repositories with specific labels.

**Why P1**: Tag filtering is already in the CLI; GUI parity is expected.

**Acceptance Criteria**:

1. WHEN the dashboard loads THEN a Tag filter dropdown SHALL appear above the Repository table showing all distinct Tags
2. WHEN the user selects a Tag THEN the table SHALL show only Repositories with that Tag
3. WHEN the user clears the filter THEN all Repositories SHALL be visible again
4. WHEN no Repositories match the selected Tag THEN an empty state message SHALL appear

**Independent Test**: Tag several Repositories via CLI, open dashboard, select a Tag filter, verify only matching Repositories appear.

---

### P1: Macro Builder Page ⭐ MVP

**User Story**: As a desktop app user, I want to define Macros with a visual step editor so that I can create multi-step workflows without using the CLI.

**Why P1**: User explicitly requested full Macro builder (D32).

**Acceptance Criteria**:

1. WHEN the user navigates to the Macros page THEN it SHALL show a list of all defined Macros with step counts
2. WHEN the user clicks "New Macro" THEN a builder form SHALL appear with name input and an empty step list
3. WHEN the user adds a step THEN they SHALL choose between Git Operation (Fetch/Pull/Checkout) and Shell Command
4. WHEN the user adds a Checkout step THEN a branch name input SHALL appear
5. WHEN the user adds a Shell Command step THEN a command text input and optional label input SHALL appear
6. WHEN the user adds a condition to a step THEN a condition text input SHALL appear
7. WHEN the user adds a rollback to a step THEN a nested step editor SHALL appear (one level deep)
8. WHEN the user toggles "Confirm" on a step THEN a confirm flag SHALL be set
9. WHEN the user saves the Macro THEN it SHALL be persisted and appear in the Macro list
10. WHEN the user defines variables THEN a key-value editor SHALL allow adding/removing variable pairs
11. WHEN the user reorders steps THEN the step order SHALL update via move up/move down buttons
12. WHEN the user edits an existing Macro THEN the builder SHALL pre-fill with the Macro's current definition
13. WHEN the user deletes a Macro THEN it SHALL be removed from the list with confirmation

**Independent Test**: Create a Macro with 3 steps (fetch, shell, pull with condition), save, verify it appears in `gitty macro list` from CLI.

---

### P1: Macro Execution & Job Monitoring ⭐ MVP

**User Story**: As a desktop app user, I want to run a Macro against a selection of Repositories and see per-Repository results so that I can execute bulk workflows from the GUI.

**Why P1**: Without execution, the Macro builder is useless.

**Acceptance Criteria**:

1. WHEN the user clicks "Run" on a Macro THEN a Selection dialog SHALL appear with options: All, Group (dropdown), Tag (dropdown), Individual (checkbox list)
2. WHEN the user confirms the selection THEN the Macro SHALL execute against the resolved Repositories
3. WHEN execution starts THEN an inline results panel SHALL appear showing each Repository's status (Pending → Running → Success/Failed/Skipped)
4. WHEN a Repository's Job succeeds THEN it SHALL show a success indicator
5. WHEN a Repository's Job fails THEN it SHALL show the error message and any rollback outcome
6. WHEN a Repository is skipped (Missing state) THEN it SHALL show the skip reason
7. WHEN all Jobs complete THEN a summary SHALL appear (N succeeded, N failed, N skipped)
8. WHEN the user dismisses the results panel THEN it SHALL close

**Independent Test**: Define a Macro via CLI, run it from the GUI against a Group, verify per-Repository results appear inline.

---

## Edge Cases

- WHEN the config file is deleted while the app is running THEN the file-watcher SHALL recreate a default config and reload
- WHEN a Group has no Repositories THEN the sidebar tree SHALL still show the Group node (empty)
- WHEN the Macro builder has zero steps THEN the "Save" button SHALL be disabled
- WHEN a Macro is running and the user navigates away THEN execution SHALL continue (fire-and-forget)
- WHEN the sidebar tree has deeply nested Groups (5+ levels) THEN indentation SHALL remain readable with scroll support
- WHEN a Repository is Missing THEN it SHALL still appear in the sidebar tree with a muted/dimmed style

---

## Requirement Traceability

| Requirement ID | Story | Phase | Status |
|----------------|-------|-------|--------|
| M4GUI-01 | P1: Tauri managed state + file-watcher | Design | Pending |
| M4GUI-02 | P1: Structured error DTOs | Design | Pending |
| M4GUI-03 | P1: Group IPC commands (7 commands) | Design | Pending |
| M4GUI-04 | P1: Tag IPC commands (3 commands) | Design | Pending |
| M4GUI-05 | P1: Macro IPC commands (5 commands) | Design | Pending |
| M4GUI-06 | P1: Sidebar Group tree component | Design | Pending |
| M4GUI-07 | P1: Groups admin page | Design | Pending |
| M4GUI-08 | P1: Repository detail — Group dropdown | Design | Pending |
| M4GUI-09 | P1: Repository detail — Tag editor | Design | Pending |
| M4GUI-10 | P1: Dashboard Tag filter | Design | Pending |
| M4GUI-11 | P1: Macro list + builder page | Design | Pending |
| M4GUI-12 | P1: Macro execution Selection dialog | Design | Pending |
| M4GUI-13 | P1: Job monitoring results panel | Design | Pending |
| M4GUI-14 | P1: Sidebar nav update (Dashboard, Groups, Macros, Settings) | Design | Pending |

**Coverage:** 14 total, 0 mapped to tasks, 14 unmapped

---

## Success Criteria

- [ ] All 7 user stories pass their acceptance criteria
- [ ] All 14 requirements mapped to tasks and verified
- [ ] `cargo test && cargo clippy -- -D warnings && cargo fmt --check` passes
- [ ] `npm run check` passes (svelte-check)
- [ ] CLI and GUI stay in sync via file-watcher (manual test)
- [ ] Thermo-nuclear code quality review passes
