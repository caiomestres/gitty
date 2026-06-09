# Activity Log

The **Activity Log** maintains a timestamped history of operations, executions, and state changes in your workspace.

## Overview

The Activity Log provides visibility into:

- Macro executions (start, completion, failures)
- Repository state changes (discovered, removed, moved)
- Health evaluations
- Scheduler runs
- Configuration changes
- System events

## Storage

The Activity Log is stored separately from Config:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\activity.json` |
| macOS | `~/Library/Application Support/gitty/activity.json` |
| Linux | `~/.config/gitty/activity.json` |

### Ring Buffer

The log uses a ring buffer (circular buffer) with a configurable maximum size:

```json
{
  "activity_log": {
    "max_entries": 1000
  }
}
```

**Default:** 1000 entries
**Range:** 100 - 10000 entries

When the buffer fills, oldest entries are automatically removed to maintain the size limit.

### Storage Format

```json
{
  "version": 1,
  "entries": [
    {
      "id": "entry-uuid",
      "timestamp": "2024-01-15T09:30:00Z",
      "type": "macro_execution",
      "severity": "info",
      "message": "Macro 'Fetch All' completed",
      "details": {
        "macro_name": "Fetch All",
        "job_id": "job-uuid",
        "repos_targeted": 15,
        "repos_succeeded": 14,
        "repos_failed": 1
      }
    }
  ]
}
```

## Entry Types

### Macro Execution

Records when macros are executed:

```json
{
  "type": "macro_execution",
  "severity": "info",
  "message": "Macro 'Morning Sync' completed",
  "details": {
    "macro_name": "Morning Sync",
    "job_id": "job-uuid",
    "selection_type": "All",
    "trigger": "manual"
  }
}
```

**Triggers:** `manual`, `scheduler`, `webhook` (future)

### Repository Event

Records repository lifecycle events:

```json
{
  "type": "repository_discovered",
  "severity": "info",
  "message": "Repository discovered: myproject",
  "details": {
    "repository_uuid": "repo-uuid",
    "repository_name": "myapp",
    "path": "/home/user/projects/myapp"
  }
}
```

**Types:**
- `repository_discovered` — New repo found during scan
- `repository_removed` — Repo unregistered
- `repository_moved` — Re-linking occurred
- `repository_missing` — Marked as missing
- `repository_relinked` — Successfully re-linked after move

### Health Event

Records health status changes:

```json
{
  "type": "health_changed",
  "severity": "warning",
  "message": "Repository 'api' health changed to warning",
  "details": {
    "repository_uuid": "repo-uuid",
    "repository_name": "api",
    "previous_status": "healthy",
    "new_status": "warning",
    "checks": ["freshness"]
  }
}
```

### Scheduler Event

Records scheduler lifecycle:

```json
{
  "type": "scheduler_run",
  "severity": "info",
  "message": "Scheduler executed macro 'Fetch All'",
  "details": {
    "macro_name": "Fetch All",
    "trigger_type": "Simple",
    "duration_seconds": 45
  }
}
```

**Types:**
- `scheduler_run` — Successful scheduled execution
- `scheduler_skipped` — Execution skipped (power policy, time window)
- `scheduler_started` — Daemon started
- `scheduler_stopped` — Daemon stopped

### Configuration Event

Records configuration changes:

```json
{
  "type": "config_changed",
  "severity": "info",
  "message": "Scan root added: /home/user/new-projects",
  "details": {
    "change_type": "scan_root_added",
    "path": "/home/user/new-projects"
  }
}
```

**Types:**
- `scan_root_added`
- `scan_root_removed`
- `group_created`
- `group_deleted`
- `macro_created`
- `macro_deleted`
- `theme_changed`

### Liveness Event

Records endpoint status changes:

```json
{
  "type": "liveness_changed",
  "severity": "error",
  "message": "Environment 'dev' for 'myapp' is now down",
  "details": {
    "repository_uuid": "repo-uuid",
    "repository_name": "myapp",
    "environment_name": "dev",
    "endpoint": "http://localhost:3000/health",
    "previous_status": "up",
    "new_status": "down",
    "response_time_ms": 5000
  }
}
```

## Severity Levels

| Level | Color | Use Case |
|-------|-------|----------|
| `debug` | Gray | Detailed diagnostic info |
| `info` | Blue | Normal operations |
| `warning` | Yellow | Attention recommended |
| `error` | Red | Operation failures |
| `critical` | Red | Serious problems |

## GUI Activity View

The **Activity** page in the sidebar provides a dedicated view:

### Filter Sidebar

- **Date range** — Today, Last 7 days, Last 30 days, Custom
- **Type** — Multi-select by entry type
- **Severity** — Multi-select by severity
- **Repository** — Filter to specific repo events

### Entry List

| Column | Description |
|--------|-------------|
| Time | Relative time (hover for absolute) |
| Severity | Color-coded icon |
| Type | Entry category |
| Message | Human-readable summary |
| Repository | Link to repo (if applicable) |

### Entry Detail

Click any entry to see:

- Full timestamp
- All detail fields
- Related entries (same job/run)
- Raw JSON (for debugging)

### Real-time Updates

The Activity Log updates in real-time:
- New entries appear automatically
- Severity icons animate on arrival
- Unread count shown on sidebar icon

## CLI Access

### View Activity

```bash
# Show recent activity
gitty activity

# Show with filter
gitty activity --type macro_execution

# Show last N entries
gitty activity --limit 50

# Show date range
gitty activity --since 2024-01-01 --until 2024-01-15

# JSON output for scripting
gitty activity --format json
```

### Configuration

```bash
# Set max entries
gitty activity config --max-entries 2000

# Clear log
gitty activity clear

# Export to file
gitty activity export --output activity-backup.json
```

## Retention & Cleanup

### Automatic

- Ring buffer enforces size limit automatically
- Oldest entries removed when limit reached
- No manual cleanup required

### Manual

```bash
# Clear all entries (irreversible)
gitty activity clear

# Compact by removing debug entries
gitty activity compact --keep-severity info,warning,error
```

### Archival

For long-term retention:

```bash
# Export before clearing
gitty activity export --output activity-$(date +%Y%m%d).json
gitty activity clear
```

## Use Cases

### Debugging Failures

1. Filter by `type: macro_execution`
2. Look for `severity: error`
3. Click entry for job details
4. See which repository failed and why

### Understanding Health Changes

1. Filter by `type: health_changed`
2. Select date range when issue started
3. Look for pattern (same check? same repo?)
4. Correlate with macro executions

### Auditing Activity

1. Filter by repository
2. Review all operations performed
3. See when it was last fetched/pulled

### Troubleshooting Scheduler

1. Filter by `type: scheduler_run` and `scheduler_skipped`
2. Check if runs are occurring as expected
3. See why executions were skipped

## Best Practices

### Buffer Size

| Use Case | Recommended Size |
|----------|-----------------|
| Light usage | 500 entries |
| Standard development | 1000 entries (default) |
| Heavy automation | 5000 entries |
| Compliance/auditing | 10000 entries |

### Review Cadence

- **Daily**: Quick scan for errors
- **Weekly**: Review health patterns
- **Monthly**: Export and clear if growing large

### Privacy

The Activity Log contains paths and repository names. Be mindful when:
- Sharing exports
- Including in bug reports (sanitize paths)
- Backing up to cloud storage

## Integration with Notifications

Activity Log entries can trigger notifications:

```json
{
  "notifications": {
    "trigger": "on_critical"
  }
}
```

When enabled, `severity: critical` entries generate notifications alongside health changes.

## Performance

- Writes are append-only (fast)
- Reads load only displayed entries (paginated)
- Memory usage bounded by buffer size
- File size typically < 1MB (1000 entries)

## Troubleshooting

### Activity Log not updating

1. Check that entries are being created (may be filtered out)
2. Verify activity.json is writable
3. Check if max_entries reached (oldest auto-removed)

### Old entries missing

- Ring buffer removed them (expected behavior)
- Increase max_entries if you need longer history

### Large file size

1. Clear old entries: `gitty activity clear`
2. Reduce max_entries
3. Check for excessive debug logging

### Corrupted activity.json

1. Back up the file
2. Delete and restart Gitty (creates fresh log)
3. Report issue with backup file

## See Also

- [Macros](macros.md) — Automated operations
- [Health](health.md) — Health monitoring
- [Scheduler](scheduler.md) — Background automation
- [Liveness](liveness.md) — Endpoint monitoring