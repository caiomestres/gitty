# Automation Commands

Automation commands manage macros, the scheduler, and background execution.

## macro

Manage and execute macros.

### macro list

List all defined macros.

```bash
gitty macro list [OPTIONS]
```

**Examples:**

```bash
# List macros
gitty macro list

# With step counts
gitty macro list --with-steps

# JSON for scripting
gitty macro list --format json
```

**Output:**

```
MACRO              STEPS   LAST RUN
───────────────────────────────────
Morning Sync       2       2 hours ago
Deploy to Staging  4       Never
Clean and Rebuild  3       3 days ago
__scheduler_default 1      (system)
```

**Options:**

| Option | Description |
|--------|-------------|
| `--with-steps` | Include step counts |
| `--with-variables` | Include variable definitions |
| `--format` | Output format |

### macro define

Create a new macro.

```bash
gitty macro define [OPTIONS] <NAME> <STEPS...>
```

**Examples:**

```bash
# Simple macro
gitty macro define "Fetch All" fetch

# Multiple steps
gitty macro define "Morning Sync" fetch pull

# With checkout
gitty macro define "Sync Main" "checkout:main" fetch pull

# With shell command
gitty macro define "Update Deps" "checkout:main" pull "shell:npm ci"

# With condition
gitty macro define "Smart Pull" "pull (if_behind and if_clean)"
```

**Step Syntax:**

| Step | Format | Example |
|------|--------|---------|
| Fetch | `fetch` | `fetch` |
| Pull | `pull` | `pull` |
| Checkout | `checkout:<branch>` | `checkout:main` |
| Shell | `shell:<command>` | `shell:make build` |
| With condition | `<step> (if_<condition>)` | `pull (if_behind)` |

**Conditions:**

| Condition | Description |
|-----------|-------------|
| `if_dirty` | Run if uncommitted changes |
| `if_clean` | Run if clean working tree |
| `if_ahead` | Run if ahead of remote |
| `if_behind` | Run if behind remote |
| `if_detached` | Run if detached HEAD |
| `if_not_on_branch:<name>` | Run if not on branch |

**Output:**

```
Created macro 'Morning Sync' with 2 steps:
  1. fetch
  2. pull
```

**Options:**

| Option | Description |
|--------|-------------|
| `--rollback <steps...>` | Define rollback steps |

**Arguments:**

| Argument | Description |
|----------|-------------|
| `NAME` | Macro name (quoted if spaces) |
| `STEPS` | One or more steps |

### macro show

Display macro details.

```bash
gitty macro show <NAME>
```

**Examples:**

```bash
# Show macro
gitty macro show "Morning Sync"
```

**Output:**

```
Macro: Morning Sync
ID: macro-uuid-here
Steps:
  1. fetch
  2. pull

Variables: None
Rollback: None

Last run: 2 hours ago
Times run: 47
```

### macro run

Execute a macro.

```bash
gitty macro run [OPTIONS] <NAME>
```

**Examples:**

```bash
# Run on all repos
gitty macro run "Morning Sync"

# Run on specific repo
gitty macro run "Morning Sync" --repo myapp

# Run on group
gitty macro run "Morning Sync" --group work

# Run on tagged repos
gitty macro run "Morning Sync" --tag favorite

# With variables
gitty macro run "Deploy" --var branch=main --var env=staging

# With confirmation
gitty macro run "Clean Slate" --confirm
```

**Output:**

```
Running macro 'Morning Sync' on 15 repositories...

Progress:
✓ myapp     Step 1: fetch ✓   Step 2: pull ✓
✓ api       Step 1: fetch ✓   Step 2: pull ✓
✗ web       Step 1: fetch ✓   Step 2: pull ✗ (merge conflict)
...

Done. 14 succeeded, 1 failed.
```

**Options:**

| Option | Description |
|--------|-------------|
| `--repo <name>` | Target specific repository |
| `--group <name>` | Target group |
| `--tag <name>` | Target tagged repositories |
| `--var <name=value>` | Set variable |
| `--confirm` | Confirm before executing |
| `--dry-run` | Show what would happen |

**Arguments:**

| Argument | Description |
|----------|-------------|
| `NAME` | Macro name to run |

### macro delete

Delete a macro.

```bash
gitty macro delete [OPTIONS] <NAME>
```

**Examples:**

```bash
# Delete macro
gitty macro delete "Old Macro"
```

**Warning:** Cannot delete built-in macros (`__scheduler_default`).

**Output:**

```
Deleted macro 'Old Macro'
```

## scheduler

Manage the background scheduler.

### scheduler start

Start the scheduler daemon.

```bash
gitty scheduler start [OPTIONS]
```

**Examples:**

```bash
# Start daemon
gitty scheduler start

# Start with specific interval
gitty scheduler start --interval 30
```

**Behavior:**

- Forks/detaches from terminal
- Creates PID file
- Runs in background
- Single instance enforced

**Output:**

```
Starting scheduler daemon...
PID: 12345
Log: ~/.config/gitty/logs/scheduler.log
```

**Options:**

| Option | Description |
|--------|-------------|
| `--interval <minutes>` | Override config interval |
| `--foreground` | Run in foreground (don't detach) |

### scheduler stop

Stop the scheduler daemon.

```bash
gitty scheduler stop
```

**Output:**

```
Stopping scheduler daemon (PID: 12345)...
Stopped.
```

### scheduler status

Show scheduler status.

```bash
gitty scheduler status
```

**Output (running):**

```
Scheduler Status
══════════════════
Status: Running (PID: 12345)
Enabled: Yes
Last run: 2024-01-15 09:30:00
Next run: 2024-01-15 10:30:00
Interval: 60 minutes
Macro: __scheduler_default
Trigger: Simple
Power policy: Run Always
```

**Output (stopped):**

```
Scheduler Status
══════════════════
Status: Stopped
Enabled: Yes
Last run: Never
Next run: When started
```

### scheduler set

Configure scheduler settings.

```bash
gitty scheduler set [OPTIONS]
```

**Examples:**

```bash
# Enable/disable
gitty scheduler set --enabled true
gitty scheduler set --enabled false

# Set interval
gitty scheduler set --interval 30

# Set power policy
gitty scheduler set --power-policy AcOnly

# Set macro
gitty scheduler set --macro-id <uuid>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--enabled <bool>` | Enable/disable scheduler |
| `--interval <minutes>` | Set check interval |
| `--power-policy <policy>` | Set power policy |
| `--macro-id <uuid>` | Set macro to run |

**Power Policies:**

| Policy | Description |
|--------|-------------|
| `RunAlways` | Execute regardless of power state |
| `AcOnly` | Only when plugged in |
| `BatteryThreshold:<n>` | Only when battery > n% |

## activity

View and manage activity log.

### activity

View recent activity.

```bash
gitty activity [OPTIONS]
```

**Examples:**

```bash
# Show recent
gitty activity

# Show more entries
gitty activity --limit 100

# Filter by type
gitty activity --type macro_execution

# Date range
gitty activity --since 2024-01-01 --until 2024-01-15

# JSON output
gitty activity --format json
```

**Output:**

```
TIME     TYPE              SEVERITY  MESSAGE
════════════════════════════════════════════════════
09:30:01 macro_execution   info      Macro 'Fetch All' completed
09:15:22 health_changed    warning   Repository 'api' health changed
09:00:00 scheduler_run     info      Scheduler executed 'Fetch All'
```

**Options:**

| Option | Description |
|--------|-------------|
| `--limit <n>` | Maximum entries to show |
| `--type <types>` | Filter by entry type (comma-separated) |
| `--severity <levels>` | Filter by severity |
| `--since <date>` | Start date |
| `--until <date>` | End date |
| `--format` | Output format |

### activity clear

Clear activity log.

```bash
gitty activity clear [OPTIONS]
```

**Examples:**

```bash
# Clear all
gitty activity clear

# Clear (no confirm)
gitty activity clear --yes
```

## Common Patterns

### Daily Automation

```bash
# Create daily sync macro
gitty macro define "Daily Sync" fetch "pull (if_behind)"

# Schedule it
gitty scheduler set --enabled true --interval 360  # 6 hours
gitty scheduler start
```

### Batch Operations

```bash
# Create and run macro
gitty macro define "Update All" "checkout:main" pull
gitty macro run "Update All" --group work --confirm
```

### Maintenance Tasks

```bash
# Clean up macro
gitty macro define "Clean" "shell:git gc" "shell:git remote prune origin"
gitty macro run "Clean"
```

### Health Monitoring

```bash
# Check health after scheduled run
gitty scheduler start
gitty activity --type health_changed --since today
```

## Cron Integration

For systems without scheduler daemon:

```cron
# crontab entry
gitty macro run "Daily Sync" 2>/dev/null
```

Or use the scheduler for persistent background execution:

```bash
# Start on login
# Add to ~/.bash_profile or similar
gitty scheduler status || gitty scheduler start
```

## See Also

- [Macros Concepts](../concepts/macros.md) — Automation theory
- [Scheduler Concepts](../concepts/scheduler.md) — Background execution
- [Activity Log](../concepts/activity.md) — Operation history