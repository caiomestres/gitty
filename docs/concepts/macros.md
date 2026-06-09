# Macros

**Macros** are reusable sequences of operations that target repository selections. They are Gitty's primary automation mechanism.

## Overview

A Macro is a named, ordered list of **Steps** that can be executed against any **Repository Selection**. Macros support:

- **Variables** — Named placeholders for values
- **Conditions** — Skip steps based on repository state
- **Rollback** — Undo steps on failure
- **Confirmations** — Require user approval before execution

## Macro Components

### Name

Unique identifier for the macro. Used when running:

```bash
gitty macro run "Morning Sync"
```

### Steps

Ordered sequence of operations. Each step has:

| Property | Description |
|----------|-------------|
| `type` | `GitOperation` or `ShellCommand` |
| `command` | The operation to execute |
| `condition` | Optional predicate for conditional execution |
| `retry` | Optional retry configuration |

### Variables

Named placeholders that can be used in step commands:

```
Name: Branch Checkout
Steps:
  1. checkout:{branch}

Variables:
  - branch (required)
```

Usage:
```bash
gitty macro run "Branch Checkout" --var branch=feature/login
```

### Rollback

Steps to execute if the main sequence fails:

```
Main: pull
Rollback: shell:git reset --hard HEAD@{1}
```

If `pull` fails, the rollback executes to restore state.

## Step Types

### Git Operations

Built-in Git operations executed via shell:

| Operation | Syntax | Description |
|-----------|--------|-------------|
| `fetch` | `fetch` | `git fetch --all` |
| `pull` | `pull` | `git pull` |
| `checkout` | `checkout:main` | `git checkout main` |

**Examples:**

```
Step: fetch
Step: pull
Step: checkout:develop
```

### Shell Commands

Arbitrary shell commands:

| Command | Syntax | Description |
|---------|--------|-------------|
| Shell | `shell:npm install` | Execute any shell command |
| With variables | `shell:npm run {script}` | Variable substitution |

**Examples:**

```
Step: shell:npm ci
Step: shell:docker-compose up -d
Step: shell:make build
```

### Variable Substitution

Use `{variable_name}` syntax:

```
Name: Deploy Script
Steps:
  1. checkout:{branch}
  2. shell:./scripts/deploy.sh {environment}

Variables:
  - branch (default: main)
  - environment (required)
```

## Conditions

Steps can be conditional based on repository state:

### Available Conditions

| Condition | Description |
|-----------|-------------|
| `if_dirty` | Run only if repository has uncommitted changes |
| `if_clean` | Run only if repository is clean |
| `if_ahead` | Run only if ahead of remote |
| `if_behind` | Run only if behind remote |
| `if_detached` | Run only if in detached HEAD state |
| `if_on_branch:{branch}` | Run only if on specific branch |

### Condition Syntax

```
Step: pull (if_behind)
Step: shell:git stash (if_dirty)
Step: checkout:main (if_not_on_branch:main)
```

### Multiple Conditions

Combine with `and`/`or` (CLI syntax):

```bash
gitty macro define "Smart Sync" "fetch" "pull (if_behind and if_clean)"
```

## Retry Configuration

Git operations can be configured to retry on transient failures:

```json
{
  "retry": {
    "max_attempts": 3,
    "backoff_seconds": 5
  }
}
```

**Retry applies to:**
- Network errors (timeout, connection reset)
- Authentication failures
- Server errors (5xx)

**Never retries:**
- Shell commands (intentional — arbitrary commands are risky)
- Conflict errors (requires manual resolution)
- Dirty worktree errors

## Repository Selection

When running a macro, you specify which repositories to target:

### Selection Types

| Selection | CLI Flag | Description |
|-----------|----------|-------------|
| `All` | (default) | Every registered repository |
| `Single` | `--repo <id>` | One specific repository |
| `Group` | `--group <name>` | All repos in a group (incl. nested) |
| `Tag` | `--tag <name>` | All repos with a specific tag |
| `Multiple` | (future) | Explicit list of repositories |

### Examples

```bash
# Run on all repositories
gitty macro run "Fetch All"

# Run on a specific repository
gitty macro run "Fetch All" --repo myproject

# Run on a group
gitty macro run "Morning Sync" --group work

# Run on tagged repositories
gitty macro run "Deploy Check" --tag production

# Combined filters
gitty macro run "Update" --group work --tag active
```

## Built-in Macros

Gitty includes several built-in macros:

| Macro | Steps | Purpose |
|-------|-------|---------|
| `__fetch_all` | `fetch` | Default scheduler action |
| `__pull_all` | `pull` | Bulk update |
| `__scheduler_default` | `fetch` | Used by scheduler if not overridden |

## Job Execution

When a macro runs, it creates a **Job** tracking execution:

### Job Lifecycle

```
Pending → Running → Success
                 └→ Failed → (Rollback) → Failed+Rollback
```

### Job Structure

```json
{
  "id": "job-uuid",
  "macro_name": "Morning Sync",
  "selection": { "type": "All" },
  "status": "running",
  "started_at": "2024-01-15T09:00:00Z",
  "per_repo": {
    "repo-uuid-1": {
      "status": "success",
      "steps_completed": 2,
      "output": "..."
    },
    "repo-uuid-2": {
      "status": "failed",
      "failed_step": 1,
      "error": "Connection timeout"
    }
  }
}
```

### Monitoring

**GUI:**
- Real-time progress panel
- Per-repository status icons
- Expandable log output
- Cancel button (where supported)

**CLI:**
```bash
# Run with progress
gitty macro run "Fetch All" --progress

# Job runs in foreground; output streams to terminal
```

## Defining Macros

### CLI Definition

```bash
# Define inline (simple macros)
gitty macro define "Fetch All" fetch

# Multiple steps
gitty macro define "Update and Build" fetch "shell:npm ci"

# With rollback
gitty macro define "Deploy" "shell:./deploy.sh" --rollback "shell:./rollback.sh"
```

### GUI Definition

1. Navigate to **Macros** page
2. Click **Create Macro**
3. Enter name
4. Add steps via visual editor:
   - Select step type (Git / Shell)
   - Enter command
   - Set condition (optional)
   - Configure retry (Git ops only)
5. Add rollback steps (optional)
6. Save

## Macro Storage

Macros are stored in `config.json`:

```json
{
  "macros": [
    {
      "id": "macro-uuid",
      "name": "Morning Sync",
      "steps": [
        { "type": "GitOperation", "command": "fetch" },
        { "type": "GitOperation", "command": "pull" }
      ],
      "variables": [],
      "rollback_steps": []
    }
  ]
}
```

## Best Practices

### Macro Naming

Use descriptive, action-oriented names:

| Good | Less Good |
|------|-----------|
| "Morning Sync" | "Macro 1" |
| "Deploy to Staging" | "Stuff" |
| "Update Dependencies" | "npm" |
| "Clean and Rebuild" | "Clean" |

### Step Ordering

Order matters — think about dependencies:

```
1. fetch          # Get latest remote info
2. checkout:main  # Switch to main branch
3. pull           # Update main
4. shell:npm ci   # Install dependencies (after code update)
```

### Safety

1. **Test on one repo first** — Use `--repo` to test before running on all
2. **Use conditions** — Don't blindly pull if there are local changes
3. **Add rollbacks** — For destructive operations
4. **Require confirmations** — For dangerous macros

### Rollback Design

Not all operations can be rolled back:

| Operation | Rollback Possible? | Strategy |
|-----------|-------------------|----------|
| `fetch` | No | Harmless, no rollback needed |
| `pull` | Partial | `git reset --hard HEAD@{1}` |
| `checkout` | Yes | `git checkout -` (previous branch) |
| `shell:rm` | No | Avoid destructive shell commands |

### Variables for Flexibility

```
Name: Feature Branch Sync
Steps:
  1. fetch
  2. checkout:{branch}
  3. pull

Variables:
  - branch (required)

# Usage:
gitty macro run "Feature Branch Sync" --var branch=feature/login
```

## Common Macro Patterns

### Daily Sync

```
Name: Daily Sync
Steps:
  1. fetch
  2. pull (if_behind and if_clean)

# Run every morning
gitty macro run "Daily Sync"
```

### Dependency Update

```
Name: Update Dependencies
Steps:
  1. checkout:main
  2. pull
  3. shell:npm update
  4. shell:git commit -am "Update dependencies" (if_dirty)
  5. shell:git push (if_dirty)
```

### Clean Slate

```
Name: Clean Slate
Steps:
  1. shell:git stash (if_dirty)
  2. checkout:main
  3. pull
  4. shell:git stash pop (if_has_stash)
```

### Multi-Project Build

```
Name: Full Build
Steps:
  1. fetch
  2. pull
  3. shell:make clean
  4. shell:make build
  5. shell:make test
```

## Troubleshooting

### Macro fails on specific repository

1. Check repository status: `gitty status --repo <id>`
2. Run macro on that repo alone to see full output
3. Check for uncommitted changes, diverged branches, etc.

### Rollback didn't work

1. Verify rollback steps are valid
2. Check that rollback is appropriate for the operation
3. Some operations inherently cannot be rolled back

### Variable not substituted

1. Verify variable name matches exactly (case-sensitive)
2. Check that variable was provided: `--var name=value`
3. Ensure braces are used: `{variable}` not `variable`

### Condition not working

1. Check condition syntax: `(if_dirty)` not `if_dirty`
2. Verify repository actually meets condition
3. Some conditions require exact state matching

## See Also

- [Scheduler](scheduler.md) — Automated macro execution
- [Repository Selection](organization.md) — Filtering targets
- [CLI Reference](../cli/automation.md) — Macro commands