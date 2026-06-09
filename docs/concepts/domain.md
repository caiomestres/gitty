# Domain Model

Gitty uses precise terminology derived from the problem domain. This page defines all domain terms used in the application, APIs, and documentation.

## Core Entities

### Workspace

A named collection of one or more **Scan Roots** whose **Repositories** are managed as a single unit. In v1, a single implicit default workspace exists.

**Properties:**
- `name` — Human-readable identifier
- `scan_roots` — List of directories to scan
- `groups` — Hierarchical group tree
- `tags` — Available tag definitions
- `macros` — Named automation sequences
- `scheduler_config` — Background automation settings

### Scan Root

A filesystem directory that Gitty recursively scans to discover Git repositories. Scan Roots are the entry points for repository discovery.

**Behavior:**
- Recursive descent into subdirectories
- Skips common non-repository directories (`.git`, `node_modules`, `target`, etc.)
- Detects moves and re-links repositories via content fingerprinting

### Repository

A local Git repository discovered by scanning. The fundamental unit Gitty operates on.

**Identity:**
- `uuid` — Stable Gitty-assigned identifier
- `path` — Current filesystem location
- `root_commit_fingerprint` — Content hash for re-linking

**State:**
- `active` — Path exists and is accessible
- `missing` — Path no longer valid (may be re-linked)

## Organization

### Group

A hierarchical organizational category for repositories. Supports arbitrary nesting (tree structure).

**Constraints:**
- A repository belongs to exactly one group
- Groups can be nested to arbitrary depth
- The `Ungrouped` group is the default for newly discovered repositories

**Operations:**
- Create, rename, delete (reassigns repos to `Ungrouped`)
- Move (reparent in hierarchy)
- Tree view (hierarchical navigation)

### Tag

An additive label attached to a repository. A repository can have zero or more tags.

**Built-in Tags:**
- `favorite` — Quick-access marking

**Custom Tags:**
- User-defined strings
- No nesting or hierarchy
- Cross-cutting (one repo can have many tags)

### Favorite

A built-in system tag for marking frequently accessed repositories. Provides quick filtering in the GUI and CLI.

## Automation

### Macro

A named, ordered sequence of **Steps** that targets a **Repository Selection**.

**Components:**
- `name` — Unique identifier
- `steps` — Ordered list of operations
- `variables` — Named placeholders for values
- `rollback` — Steps to execute on failure

### Step

An individual operation within a Macro.

**Types:**
- `GitOperation` — `fetch`, `pull`, `checkout`
- `ShellCommand` — Arbitrary shell execution

**Properties:**
- `condition` — Optional predicate for conditional execution
- `retry` — Optional retry configuration (Git operations only)
- `confirmation` — Optional user prompt before execution

### Repository Selection

A set of repositories targeted by a Macro execution.

**Variants:**
- `All` — Every registered repository
- `Single` — One specific repository
- `Group` — All repositories in a group (including nested)
- `Tag` — All repositories with a specific tag
- `Multiple` — Explicit list of repositories

### Job

A single execution of a Macro against a Repository Selection.

**Lifecycle:**
1. `pending` — Queued for execution
2. `running` — Currently executing
3. `success` — All steps completed
4. `failed` — One or more steps failed (rollback may have executed)

## Health & Monitoring

### Health Check

An evaluation of a single repository against a specific criterion.

**Built-in Checks:**
| Check | Description | Severity |
|-------|-------------|----------|
| `Freshness` | Last fetch/pull age | Configurable |
| `Divergence` | Commits ahead/behind remote | Configurable |
| `Dirty` | Uncommitted changes | Warning |
| `Detached` | Not on a branch | Warning |

**Result:**
- `healthy` — Within normal parameters
- `warning` — Attention recommended
- `critical` — Action required

### Workspace Health

An aggregate score representing the overall health of the workspace.

**Calculation:**
```
score = (repos_not_critical / total_active_repos) × 100
```

Missing repositories are excluded from the calculation.

### Health Score

A percentage from 0-100 representing workspace health. Displayed prominently in the dashboard and health view.

### Liveness

HTTP endpoint monitoring for repositories with associated services. Tracks whether configured endpoints are reachable.

**Components:**
- `probe` — HTTP request to an endpoint
- `endpoint` — URL to monitor (e.g., `http://localhost:3000/health`)
- `status` — `up`, `down`, or `unknown`

**Note:** Liveness is independent from Health — a repository can be healthy (Git state is good) while its service is down.

### Activity Log

Timestamped history of operations and events.

**Entry Types:**
- Macro execution
- Health evaluation
- Repository state change
- Scheduler run
- System event

**Storage:**
- Ring buffer with configurable limit (default: 1000 entries)
- Separate from Config (`activity.json`)

## Scheduler

### Trigger

A condition that causes the scheduler to run a Macro.

**Types:**
- `Simple` — Fixed interval
- `Advanced` — Interval with time window and day constraints

### Power Policy

Scheduler behavior regarding power source.

**Options:**
- `RunAlways` — Execute regardless of power state
- `AcOnly` — Skip when on battery
- `BatteryThreshold` — Skip when battery below threshold

### Notification

An alert delivered to the user about workspace events.

**Triggers:**
- `on_critical` — Critical health status changes
- `on_any_change` — Any health status change
- `on_scheduler_complete` — After scheduled run
- `disabled` — No notifications

**Delivery:**
- In-app notification panel
- OS-native toast (where supported)

## Configuration

### Config

The main configuration file storing all Gitty settings.

**Location:**
- Windows: `%APPDATA%\gitty\config.json`
- macOS: `~/Library/Application Support/gitty/config.json`
- Linux: `~/.config/gitty/config.json`

**Schema:**
- Versioned (v1 in current release)
- JSON format
- File-level locking for concurrent access

### Theme

A visual design token set applied to the GUI.

**Bundled Themes:**
- `default` — Warm cream canvas with Cursor Orange accents
- `dark` — Dark mode with luminous highlights
- `world-cup-brasil` — Brasil national colors (green, yellow, blue)

### Lock

File-level synchronization mechanism preventing concurrent modifications.

**Types:**
- Repository lock — Per-repo PID-based lock during operations
- Config lock — Global lock during config updates

## See Also

- [Repository](repository.md) — UUIDs, re-linking, and identity
- [Organization](organization.md) — Groups and Tags in depth
- [Health](health.md) — Health checks and scoring
- [Macros](macros.md) — Automation and scripting
- [Scheduler](scheduler.md) — Background automation