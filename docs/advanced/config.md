# Configuration

Gitty stores configuration in a JSON file. This page documents the configuration format and advanced editing.

## Config Location

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\config.json` |
| macOS | `~/Library/Application Support/gitty/config.json` |
| Linux | `~/.config/gitty/config.json` |

## Config Structure

```json
{
  "version": 1,
  "workspace": {
    "name": "default",
    "scan_roots": [],
    "repositories": [],
    "groups": [],
    "macros": [],
    "tags": []
  },
  "scheduler": {
    "enabled": true,
    "trigger": {
      "type": "Simple",
      "interval_minutes": 60
    },
    "macro_id": "__scheduler_default",
    "power_policy": {
      "type": "RunAlways"
    }
  },
  "notifications": {
    "trigger": "on_critical"
  },
  "theme": "default",
  "health": {
    "freshness_days_warning": 3,
    "freshness_days_critical": 7,
    "divergence_ahead_warning": 5,
    "divergence_ahead_critical": 20,
    "divergence_behind_warning": 10,
    "divergence_behind_critical": 50,
    "dirty_is_warning": true,
    "detached_is_warning": true
  },
  "liveness": {
    "enabled": true,
    "default_interval_seconds": 60,
    "notification_on_failure": false
  },
  "activity_log": {
    "max_entries": 1000
  }
}
```

## Schema Reference

### version

Configuration file format version. Current: `1`.

### workspace

#### scan_roots

Array of scan root paths:

```json
{
  "workspace": {
    "scan_roots": [
      "/home/user/projects",
      "/home/user/experiments"
    ]
  }
}
```

#### repositories

Array of registered repositories:

```json
{
  "workspace": {
    "repositories": [
      {
        "uuid": "550e8400-e29b-41d4-a716-446655440000",
        "name": "myapp",
        "path": "/home/user/projects/myapp",
        "group_id": "work/mobile",
        "tags": ["favorite", "active"],
        "environments": [
          {
            "name": "dev",
            "endpoint": "http://localhost:3000/health",
            "interval_seconds": 60
          }
        ],
        "root_commit_fingerprint": "abc123..."
      }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | string | Stable repository identifier |
| `name` | string | Display name (directory) |
| `path` | string | Filesystem path |
| `group_id` | string | Assigned group path |
| `tags` | string[] | Assigned tags |
| `environments` | object[] | Liveness endpoints |
| `root_commit_fingerprint` | string | Content hash for re-linking |

#### groups

Array of group definitions:

```json
{
  "workspace": {
    "groups": [
      {
        "id": "work",
        "name": "work",
        "parent_id": null
      },
      {
        "id": "work-mobile",
        "name": "mobile",
        "parent_id": "work"
      }
    ]
  }
}
```

#### macros

Array of macro definitions:

```json
{
  "workspace": {
    "macros": [
      {
        "id": "macro-uuid",
        "name": "Morning Sync",
        "steps": [
          {
            "type": "GitOperation",
            "command": "fetch"
          },
          {
            "type": "GitOperation",
            "command": "pull",
            "condition": "if_behind"
          }
        ],
        "variables": [],
        "rollback_steps": []
      }
    ]
  }
}
```

### scheduler

```json
{
  "scheduler": {
    "enabled": true,
    "trigger": {
      "type": "Simple",
      "interval_minutes": 60
    },
    "macro_id": "__scheduler_default",
    "power_policy": {
      "type": "RunAlways"
    }
  }
}
```

**Trigger types:**

**Simple:**
```json
{
  "type": "Simple",
  "interval_minutes": 60
}
```

**Advanced:**
```json
{
  "type": "Advanced",
  "interval_minutes": 30,
  "time_window": {
    "start": "09:00",
    "end": "18:00"
  },
  "days": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
}
```

**Power policies:**

- `"RunAlways"`
- `"AcOnly"`
- `{"type": "BatteryThreshold", "threshold_percent": 25}`

### notifications

```json
{
  "notifications": {
    "trigger": "on_critical"
  }
}
```

Values: `"on_critical"`, `"on_any_change"`, `"on_scheduler_complete"`, `"disabled"`

### theme

```json
{
  "theme": "dark"
}
```

Values: `"default"`, `"dark"`, `"world-cup-brasil"`

### health

Health check thresholds:

```json
{
  "health": {
    "freshness_days_warning": 3,
    "freshness_days_critical": 7,
    "divergence_ahead_warning": 5,
    "divergence_ahead_critical": 20,
    "divergence_behind_warning": 10,
    "divergence_behind_critical": 50,
    "dirty_is_warning": true,
    "detached_is_warning": true
  }
}
```

### liveness

Liveness probe defaults:

```json
{
  "liveness": {
    "enabled": true,
    "default_interval_seconds": 60,
    "notification_on_failure": false
  }
}
```

### activity_log

```json
{
  "activity_log": {
    "max_entries": 1000
  }
}
```

## Manual Editing

**Warning:** Manual config editing is not recommended. Use GUI or CLI instead.

If you must edit manually:

1. **Close Gitty** — Both GUI and CLI
2. **Backup** — Copy config.json
3. **Edit** — Use JSON-aware editor
4. **Validate** — Check syntax (JSON validators)
5. **Restart** — Verify Gitty loads

### Validation

Gitty validates config on load:

| Issue | Behavior |
|-------|----------|
| Invalid JSON | Error, refuses to start |
| Unknown version | Warning, attempts migration |
| Missing fields | Uses defaults |
| Invalid values | Error for critical fields, warning for others |
| Schema mismatch | Best-effort loading, may crash |

## Migration

### Version 1

Current version. No migrations needed.

### Future Versions

Gitty will:
1. Detect version mismatch
2. Attempt automatic migration
3. Backup original
4. Report result

## Environment Variables

Override config location:

```bash
export GITTY_CONFIG_DIR=/custom/path
gitty list
```

Config loads from `$GITTY_CONFIG_DIR/config.json`.

## Backup and Restore

### Backup

```bash
# Backup config
cp ~/.config/gitty/config.json ~/gitty-config-backup-$(date +%Y%m%d).json

# Or via CLI
gitty config export --output backup.json
```

### Restore

```bash
# Restore config
cp backup.json ~/.config/gitty/config.json

# Or via CLI
gitty config import backup.json
```

### Sync Between Machines

Copy `config.json` to sync workspace setup:

```bash
# After setting up first machine
scp ~/.config/gitty/config.json other-machine:~/.config/gitty/

# Rescan on second machine to discover repos at new paths
gitty scan ~
```

## Troubleshooting

### Config not loading

1. Check JSON syntax: `python -m json.tool config.json`
2. Verify file permissions
3. Check Gitty logs for parse errors

### Corrupted config

1. Restore from backup
2. Or delete and recreate (lose settings, not repos)
3. Re-scan to re-discover repositories

### Migration failed

1. Check backup file exists
2. Restore manually
3. Update to newer Gitty version
4. Report issue with backup file

## See Also

- [Settings](../gui/settings.md) — GUI configuration
- [CLI Config](config.md) — Command-line reference
- [Privacy](../privacy.md) — Data handling