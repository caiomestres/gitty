# M4 GUI Design

**Spec**: `.specs/features/m4-gui/spec.md`
**Status**: Draft

---

## Architecture Overview

The M4 GUI adds ~15 new Tauri IPC commands, refactors the existing 10 to use managed state + structured errors, and builds 5 new frontend pages/components.

```
Frontend (SvelteKit SPA)
├── Sidebar.svelte (rewritten → GroupTree + nav)
├── +page.svelte (enhanced → Tag filter)
├── /repo/[id]/+page.svelte (enhanced → Group dropdown + Tag editor)
├── /groups/+page.svelte (new → Group CRUD admin)
├── /macros/+page.svelte (new → Macro list + builder)
└── lib/types/workspace.ts (extended → new DTOs)

         │ invoke()
         ▼
Tauri IPC Layer (src-tauri/src/lib.rs → split into modules)
├── state.rs (new → AppState, file-watcher, config reload)
├── error.rs (new → ErrorDto, error mapping)
├── commands/workspace.rs (refactored → existing 10 commands)
├── commands/groups.rs (new → 7 Group commands)
├── commands/tags.rs (new → 3 Tag commands)
└── commands/macros.rs (new → 5 Macro commands)

         │ depends on
         ▼
gitty-core (unchanged — all domain logic already exists)
```

---

## Code Reuse Analysis

### Existing Components to Leverage

| Component | Location | How to Use |
|-----------|----------|------------|
| Group CRUD | `crates/gitty-core/src/group.rs` | Call directly from Tauri commands |
| Tag add/remove/filter | `crates/gitty-core/src/tag.rs` | Call directly from Tauri commands |
| MacroDef CRUD | `crates/gitty-core/src/macro_def.rs` | Call directly from Tauri commands |
| Macro execution | `crates/gitty-core/src/execution.rs` | Call `execute_macro()` from Tauri |
| Selection resolver | `crates/gitty-core/src/selection.rs` | Resolve targets in `run_macro` command |
| Config load/save | `crates/gitty-core/src/config/` | Used by managed state |
| Existing DTOs | `src-tauri/src/lib.rs` | Refactor into modules, extend |
| Existing CSS tokens | `src/lib/styles/tokens.css` | All new components use existing tokens |
| Existing button styles | `src/routes/+page.svelte` | Extract shared `.btn-*` classes |

### Integration Points

| System | Integration Method |
|--------|-------------------|
| `notify` crate (new dep) | File-watcher on config path → reload `Mutex<Config>` |
| Tauri managed state | `State<'_, AppState>` parameter on all commands |
| Tauri event system | `app_handle.emit("config-changed", ())` on file change |
| Frontend event listener | `listen("config-changed", callback)` → reload data |

---

## Components

### AppState (Rust — new)

- **Purpose**: Centralized config cache with file-watcher
- **Location**: `src-tauri/src/state.rs`
- **Interfaces**:
  - `AppState::new(config: Config, config_path: PathBuf) -> Self`
  - `AppState::config(&self) -> MutexGuard<Config>` — read access
  - `AppState::with_config_write<F, T>(&self, f: F) -> Result<T, AppError>` — write + auto-save
  - `AppState::reload(&self) -> Result<(), AppError>` — reload from disk (called by watcher)
- **Dependencies**: `gitty_core::Config`, `notify`, `std::sync::Mutex`
- **Internal**: `Mutex<Config>`, `PathBuf` (config file path)

### ErrorDto (Rust — new)

- **Purpose**: Structured error responses for all IPC commands
- **Location**: `src-tauri/src/error.rs`
- **Interfaces**:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}
```

Error codes: `group_not_found`, `repository_not_found`, `macro_not_found`, `duplicate_group_name`, `duplicate_macro_name`, `cannot_delete_default_group`, `cycle_detected`, `empty_tag`, `config_error`, `git_error`, `io_error`, `unknown`

- **Dependencies**: Maps from `gitty_core::CoreError`
- **Reuses**: `CoreError` enum variants for classification

### Group IPC Commands (Rust — new)

- **Purpose**: Expose Group CRUD to frontend
- **Location**: `src-tauri/src/commands/groups.rs`
- **Commands**:
  - `list_groups() -> Vec<GroupDto>`
  - `create_group(name: String, parent_id: Option<String>) -> GroupDto`
  - `rename_group(id: String, new_name: String) -> ()`
  - `delete_group(id: String) -> ()`
  - `move_group(id: String, new_parent_id: Option<String>) -> ()`
  - `assign_repo_to_group(repo_id: String, group_id: String) -> ()`
  - `group_tree() -> Vec<GroupTreeNodeDto>`
- **DTOs**:

```rust
#[derive(Serialize)]
struct GroupDto {
    id: String,
    name: String,
    parent_id: Option<String>,
    repo_count: usize,
}

#[derive(Serialize)]
struct GroupTreeNodeDto {
    group: GroupDto,
    children: Vec<GroupTreeNodeDto>,
    repos: Vec<RepoDto>,
}
```

### Tag IPC Commands (Rust — new)

- **Purpose**: Expose Tag management to frontend
- **Location**: `src-tauri/src/commands/tags.rs`
- **Commands**:
  - `list_tags() -> Vec<TagDto>`
  - `add_tag(repo_id: String, tag: String) -> ()`
  - `remove_tag(repo_id: String, tag: String) -> ()`
- **DTOs**:

```rust
#[derive(Serialize)]
struct TagDto {
    name: String,
    repo_count: usize,
}
```

### Macro IPC Commands (Rust — new)

- **Purpose**: Expose Macro CRUD + execution to frontend
- **Location**: `src-tauri/src/commands/macros.rs`
- **Commands**:
  - `list_macros() -> Vec<MacroDto>`
  - `get_macro(name_or_id: String) -> MacroDto`
  - `define_macro(name: String, steps: Vec<StepDto>, variables: HashMap<String, String>) -> MacroDto`
  - `delete_macro(id: String) -> ()`
  - `run_macro(name_or_id: String, selection: SelectionDto) -> Vec<JobDto>`
- **DTOs**:

```rust
#[derive(Serialize, Deserialize)]
struct MacroDto {
    id: String,
    name: String,
    steps: Vec<StepDto>,
    variables: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct StepDto {
    kind: StepKindDto,
    condition: Option<String>,
    rollback: Option<Box<StepDto>>,
    confirm: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum StepKindDto {
    #[serde(rename = "git_op")]
    GitOp { op: String, branch: Option<String> },
    #[serde(rename = "shell")]
    Shell { command: String, label: Option<String> },
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum SelectionDto {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "group")]
    Group { id: String },
    #[serde(rename = "tag")]
    Tag { name: String },
    #[serde(rename = "multiple")]
    Multiple { ids: Vec<String> },
}

#[derive(Serialize)]
struct JobDto {
    id: String,
    repo_id: String,
    repo_name: String,
    status: String,
    error: Option<String>,
    step_results: Vec<StepResultDto>,
}

#[derive(Serialize)]
struct StepResultDto {
    step_index: usize,
    status: String,
    output: Option<String>,
}
```

### Sidebar GroupTree (Svelte — rewrite)

- **Purpose**: VS Code-style collapsible tree showing Groups → Repositories
- **Location**: `src/lib/components/Sidebar.svelte` (rewrite)
- **Props**: None (fetches own data via `invoke`)
- **State**: `$state` for tree data, collapsed node set
- **Behavior**: Listens to `config-changed` Tauri event to reload tree data
- **Reuses**: Existing sidebar brand/nav styling, CSS tokens

### GroupsPage (Svelte — new)

- **Purpose**: CRUD admin panel for Groups
- **Location**: `src/routes/groups/+page.svelte`
- **Features**: List all Groups (flat, with parent name + repo count), Create dialog, Rename inline, Delete with confirmation, Move (parent selector)
- **Reuses**: Dialog pattern from `+page.svelte` (scan dialog), button styles

### MacrosPage (Svelte — new)

- **Purpose**: Macro list + visual builder + execution
- **Location**: `src/routes/macros/+page.svelte`
- **Features**: List Macros, New/Edit builder form, Delete with confirmation, Run button → Selection dialog → Results panel
- **Sub-components** (inline or extracted as needed):
  - StepEditor: add/remove/reorder steps, type selector, Git Op fields, Shell fields, condition, rollback, confirm
  - VariableEditor: key-value pair list
  - SelectionDialog: radio (All/Group/Tag/Individual) + appropriate picker
  - JobResultsPanel: per-Repository status list + summary

### Enhanced Dashboard (Svelte — modify)

- **Purpose**: Add Tag filter dropdown above repo table
- **Location**: `src/routes/+page.svelte` (modify)
- **Changes**: Add `list_tags` invoke, render dropdown, filter displayed repos client-side

### Enhanced RepoDetail (Svelte — modify)

- **Purpose**: Add Group dropdown + Tag editor
- **Location**: `src/routes/repo/[id]/+page.svelte` (modify)
- **Changes**: Add Group selector (loads `list_groups`, shows current, calls `assign_repo_to_group`), Tag editor (inline list + add input, calls `add_tag`/`remove_tag`)

---

## Data Models

### Frontend TypeScript DTOs (new/extended)

```typescript
interface ErrorDto {
  code: string;
  message: string;
}

interface GroupDto {
  id: string;
  name: string;
  parent_id: string | null;
  repo_count: number;
}

interface GroupTreeNodeDto {
  group: GroupDto;
  children: GroupTreeNodeDto[];
  repos: RepoDto[];
}

interface TagDto {
  name: string;
  repo_count: number;
}

interface MacroDto {
  id: string;
  name: string;
  steps: StepDto[];
  variables: Record<string, string>;
}

interface StepDto {
  kind: StepKindDto;
  condition: string | null;
  rollback: StepDto | null;
  confirm: boolean;
}

type StepKindDto =
  | { type: 'git_op'; op: string; branch?: string }
  | { type: 'shell'; command: string; label?: string };

type SelectionDto =
  | { kind: 'all' }
  | { kind: 'group'; id: string }
  | { kind: 'tag'; name: string }
  | { kind: 'multiple'; ids: string[] };

interface JobDto {
  id: string;
  repo_id: string;
  repo_name: string;
  status: string;
  error: string | null;
  step_results: StepResultDto[];
}

interface StepResultDto {
  step_index: number;
  status: string;
  output: string | null;
}
```

---

## Error Handling Strategy

| Error Scenario | Handling | User Impact |
|----------------|----------|-------------|
| Config file missing | File-watcher recreates default, reloads | Brief flash, empty workspace |
| Group not found (stale ID) | `AppError { code: "group_not_found", ... }` | Toast/banner with message |
| Duplicate Group name | `AppError { code: "duplicate_group_name", ... }` | Validation error on form |
| Cannot delete Ungrouped | `AppError { code: "cannot_delete_default_group", ... }` | Error message in delete dialog |
| Cycle detected on Group move | `AppError { code: "cycle_detected", ... }` | Error message in move dialog |
| Macro execution failure (per-repo) | Job with `failed` status + error string | Red indicator per Repository in results panel |
| File-watcher error (notify) | Log warning, continue with stale state | User sees stale data until manual refresh |

---

## Tech Decisions (only non-obvious ones)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Split `lib.rs` into modules | `state.rs`, `error.rs`, `commands/*.rs` | Current 389-line monolith will grow to ~800+ with new commands; module split prevents file-size explosion |
| `notify` for file-watching | `notify` crate v7+ with debounce | Mature, cross-platform, used by Tauri itself internally |
| Client-side Tag filtering | Filter in frontend after loading all repos | Repo count is small enough (<1000); avoids a new IPC command for filtered lists |
| Step reordering via move up/down | Buttons instead of drag-and-drop | Much simpler to implement; drag-and-drop can be added later |
| Macro edit = delete + redefine | No in-place Macro update in core | `MacroDef` has no `update_macro` method; delete + redefine with same name is equivalent |
