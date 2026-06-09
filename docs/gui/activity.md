# Activity Log View

The **Activity Log** page provides a filterable, chronological history of all workspace operations, state changes, and events.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Activity                                          [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┬──────────────────────────────────────────┐│
│  │          │  Filter: [All types ▼]  [All severity ▼]   ││
│  │ Filters  │  Date: [Last 7 days ▼]  [Repo: All ▼]      ││
│  │          │                                           ││
│  │ □ Macro  │  [Search...]                      [Refresh]││
│  │ □ Health │                                           ││
│  │ □ Repo   │  ─────────────────────────────────────────││
│  │ □ Sched  │                                           ││
│  │          │  🟢 10:30  Macro 'Fetch All' completed   ││
│  │          │      15 repos targeted, 15 succeeded      ││
│  │          │                                           ││
│  │ Severity │  🟢 10:15  Repository 'myapp' discovered   ││
│  │          │      Path: /home/user/projects/myapp      ││
│  │ ☑ Info   │                                           ││
│  │ ☑ Warn   │  🟡 09:45  Health changed to warning     ││
│  │ ☑ Error  │      Repository: api                      ││
│  │          │      Check: Freshness (4 days stale)       ││
│  │          │                                           ││
│  │          │  🟢 09:30  Scheduler executed macro       ││
│  │          │      'Fetch All' — 14/15 succeeded         ││
│  │          │                                           ││
│  └──────────┴──────────────────────────────────────────┘│
│                                                             │
│  Showing 25 of 127 entries    [1] [2] [3] [Next]           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Sidebar Filters

The left sidebar provides multi-select filters:

### By Type

| Type | Description |
|------|-------------|
| **Macro** | Macro executions and job completions |
| **Health** | Health evaluation results and changes |
| **Repository** | Discovery, removal, moves, re-links |
| **Scheduler** | Scheduler runs, starts, stops, skips |
| **Liveness** | Endpoint status changes |
| **Config** | Settings changes |

Select multiple types with checkboxes.

### By Severity

| Severity | Color | Meaning |
|----------|-------|---------|
| **Info** | 🟢 | Normal operations |
| **Warning** | 🟡 | Attention recommended |
| **Error** | 🔴 | Operation failures |
| **Critical** | 🔴 | Serious problems |

## Top Bar Filters

### Date Range

Quick selections:

- **Today** — Since midnight
- **Last 24 hours** — Rolling window
- **Last 7 days** — Past week
- **Last 30 days** — Past month
- **All time** — Complete log
- **Custom** — Pick start/end dates

### Repository

Filter to specific repository events:

```
Repository: [All ▼]
          ├─ All repositories
          ├─ ────────────────
          ├─ myapp ★
          ├─ api
          ├─ web
          └─ docs
```

### Search

Text search across:
- Entry message
- Details fields
- Repository names
- Macro names

Real-time filtering as you type.

## Entry List

### Entry Format

```
🟢 10:30:45  Macro 'Fetch All' completed
    └─ 15 repos targeted, 15 succeeded, 0 failed
    [myapp] [api] [web] ... [+12 more]
```

**Elements:**

| Element | Description |
|---------|-------------|
| **Icon** | Severity color |
| **Time** | Relative (hover for absolute) |
| **Message** | Human-readable summary |
| **Details** | Expandable additional info |
| **Tags** | Related entities (clickable) |

### Severity Icons

| Icon | Severity |
|------|----------|
| 🔵 | Debug (rarely shown) |
| 🟢 | Info |
| 🟡 | Warning |
| 🔴 | Error |
| 🔴 | Critical |

### Entry Types Display

#### Macro Execution

```
🟢 Macro 'Morning Sync' completed
   Trigger: scheduler
   Duration: 45 seconds
   Repositories: 15
   Result: 14 succeeded, 1 failed
   
   [View Job Details]
```

#### Health Changed

```
🟡 Health status changed to warning
   Repository: api
   Previous: healthy
   New: warning
   Checks: freshness (3 days stale)
   
   [View Repository]
```

#### Repository Discovered

```
🟢 Repository discovered
   Name: myapp
   UUID: 550e8400-e29b-41d4-a716-446655440000
   Path: /home/user/projects/myapp
   Group: Ungrouped
   
   [View Repository]
```

#### Scheduler Event

```
🟢 Scheduler executed macro 'Fetch All'
   Trigger: Simple (60 minute interval)
   Repositories: 15
   Result: 14 succeeded, 1 failed
   
   [View Details]
```

#### Liveness Changed

```
🔴 Liveness endpoint down
   Repository: myapp
   Environment: dev
   Endpoint: http://localhost:3000/health
   Previous: up
   New: down
   Response time: 5000ms (timeout)
   
   [View Repository]
```

## Entry Detail

Click any entry to expand:

```
┌─────────────────────────────────────────────────────────┐
│ 🟢 Macro 'Fetch All' completed                          │
│ ════════════════════════════════════════════════════════│
│                                                         │
│  Timestamp: 2024-01-15 10:30:45 UTC                   │
│  Entry ID: entry-uuid-here                              │
│  Type: macro_execution                                  │
│  Severity: info                                         │
│                                                         │
│  ─────────────────────────────────────────────────────  │
│                                                         │
│  Macro Name: Fetch All                                  │
│  Job ID: job-uuid-here                                  │
│  Trigger: manual                                        │
│  Selection Type: All                                    │
│  Duration: 45 seconds                                   │
│                                                         │
│  Results:                                               │
│  ├─ Total: 15 repositories                             │
│  ├─ Succeeded: 15                                       │
│  └─ Failed: 0                                           │
│                                                         │
│  [View Full Job Log]  [Export as JSON]                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Detail Sections

| Section | Content |
|---------|---------|
| **Metadata** | Entry ID, timestamp, type, severity |
| **Type-specific** | Fields vary by entry type |
| **Results** | Outcome summary |
| **Actions** | Links to related views |

## Real-Time Updates

The Activity Log updates automatically:

- **New entries** — Fade in at top
- **Badge count** — Unread count on sidebar icon
- **Sound** — Optional notification sound (configurable)

### Unread Indicator

```
Activity  🔴 3
```

Click to mark as read, or auto-mark after 5 seconds viewing.

## Empty States

### No Entries

```
No activity recorded yet.

Operations, macro executions, and state changes
will appear here as they happen.
```

### No Matching Entries

```
No entries match your current filters.

Try:
• Expanding the date range
• Selecting different entry types
• Clearing the search
[Clear All Filters]
```

### Filtered Out All

```
All 127 entries are filtered out.

Current filters:
• Type: Macro only
• Severity: Critical only
• Date: Today

No critical macro executions today.
[Clear Filters]
```

## Pagination

Standard pagination controls:

```
Showing 1-25 of 127 entries    [1] [2] [3] [4] [5] [Next] [Last]
```

Options:
- Items per page: 25, 50, 100
- Jump to first/last
- Previous/next

## Export

Export filtered or all entries:

```
[Export ▼]
    ├─ Export visible (JSON)
    ├─ Export all matching (JSON)
    ├─ Export all time (JSON)
    └─ Export visible (CSV)
```

**Privacy Note:** Exports include paths and repository names. Review before sharing.

## Best Practices

### Daily Review

1. Check for errors (red entries)
2. Review warning trends (yellow)
3. Verify scheduler is running
4. Note any unexpected changes

### Debugging

1. Filter to relevant repository
2. Expand time range to capture context
3. Look for patterns (repeated failures)
4. Check job details for full output

### Auditing

1. Export logs before clearing
2. Keep exports in version control (if appropriate)
3. Review periodically for unexpected access

## CLI Equivalent

```bash
# View recent activity
gitty activity

# Filter by type
gitty activity --type macro_execution,health_changed

# Date range
gitty activity --since 2024-01-01 --until 2024-01-15

# Limit
gitty activity --limit 100

# JSON for scripting
gitty activity --format json
```

## Troubleshooting

### Log not updating

1. Check filters aren't excluding new entries
2. Verify activity.json is writable
3. Check available disk space
4. Review max_entries setting

### Missing old entries

- Ring buffer removed them (expected)
- Increase max_entries for longer retention
- Export before clearing for archives

### Export fails

1. Check file permissions
2. Verify disk space
3. Try smaller date range
4. Check for corrupted entries

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Refresh |
| `F` | Focus search |
| `T` | Toggle type filter |
| `S` | Toggle severity filter |
| `E` | Export dialog |
| `Esc` | Clear filters / close detail |

## See Also

- [Activity Log Concepts](../concepts/activity.md) — How logging works
- [Changes View](changes.md) — Git commit history
- [Scheduler](../concepts/scheduler.md) — Background automation
- [Macros](../concepts/macros.md) — Automated operations