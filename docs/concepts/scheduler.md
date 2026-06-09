# Scheduler

The **Scheduler** is Gitty's background automation engine. It runs macros on configured triggers without requiring manual initiation.

## Overview

The scheduler periodically evaluates trigger conditions and executes macros when they are met. It runs as:

- **GUI**: A tokio task in the Tauri process
- **CLI**: A daemonized background process

## Scheduler Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Trigger   │────▶│   Decision   │────▶│   Execute   │
│   Config    │     │   Engine     │     │   Macro     │
└─────────────┘     └──────────────┘     └─────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Power State │
                    │   Battery    │
                    └──────────────┘
```

The scheduler loop:

1. Sleep until next tick (default: 30 seconds)
2. Check if a macro should run (time + power conditions)
3. If yes, execute the macro
4. Evaluate health (post-execution)
5. Generate notifications (if triggered)
6. Update state (last_run, next_run)
7. Repeat

## Trigger Types

### Simple Trigger

Runs a macro at fixed intervals:

```json
{
  "scheduler": {
    "enabled": true,
    "trigger": {
      "type": "Simple",
      "interval_minutes": 60
    },
    "macro_id": "__scheduler_default"
  }
}
```

**Behavior:**
- Runs every `interval_minutes` from scheduler start
- First run after interval elapses
- Continues indefinitely

### Advanced Trigger

Runs a macro on an interval but only within specific time windows and days:

```json
{
  "scheduler": {
    "enabled": true,
    "trigger": {
      "type": "Advanced",
      "interval_minutes": 30,
      "time_window": {
        "start": "09:00",
        "end": "18:00"
      },
      "days": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
    },
    "macro_id": "__scheduler_default"
  }
}
```

**Behavior:**
- Evaluates every `interval_minutes`
- Only executes if current time is within `time_window`
- Only executes on specified `days`
- If window is missed, waits for next valid slot

**Time Windows:**

- `start` and `end` are 24-hour format (`HH:MM`)
- Can wrap midnight: `start: "22:00"`, `end: "02:00"` (night shift mode)
- Inclusive of start, exclusive of end

**Day Names:**

Use full English day names: `Monday`, `Tuesday`, `Wednesday`, `Thursday`, `Friday`, `Saturday`, `Sunday`

## Power-Aware Scheduling

The scheduler can adjust behavior based on power source:

### Power Policies

| Policy | Behavior |
|--------|----------|
| `RunAlways` | Execute regardless of power state |
| `AcOnly` | Skip execution when on battery |
| `BatteryThreshold` | Skip when battery below threshold% |

### Configuration

```json
{
  "scheduler": {
    "enabled": true,
    "power_policy": {
      "type": "AcOnly"
    },
    "trigger": {
      "type": "Simple",
      "interval_minutes": 60
    }
  }
}
```

With threshold:

```json
{
  "scheduler": {
    "power_policy": {
      "type": "BatteryThreshold",
      "threshold_percent": 25
    }
  }
}
```

**Battery Detection:**

- Uses native platform APIs via `sysinfo` crate
- Reports "AC" when no battery detected (desktops)
- Reports battery level when available

## Default Action

The scheduler's default action is the built-in `__scheduler_default` macro:

```
Name: __scheduler_default
Steps:
  1. fetch
```

This performs a `git fetch --all` on all repositories — a safe, non-destructive operation that keeps remote information current.

### Custom Scheduler Macro

You can configure any macro as the scheduler action:

```bash
# Define a custom macro
gitty macro define "Daily Sync" fetch pull

# Get the macro ID from the list
gitty macro list

# Configure scheduler to use it
gitty scheduler set --macro-id <uuid>
```

## Scheduler State

The scheduler tracks its state in Config:

```json
{
  "scheduler": {
    "enabled": true,
    "last_run": "2024-01-15T09:30:00Z",
    "next_run": "2024-01-15T10:30:00Z"
  }
}
```

### State Fields

| Field | Description |
|-------|-------------|
| `enabled` | Whether scheduler is active |
| `last_run` | ISO 8601 timestamp of last execution |
| `next_run` | Predicted next execution time |

## Scheduler Lifecycle

### GUI Mode

When Gitty GUI launches:

1. Read scheduler config from Config
2. If enabled, spawn tokio task
3. Task loops with 30-second sleep
4. On trigger match, execute macro
5. Task terminates when GUI closes

**Note:** Scheduler only runs while GUI is open.

### CLI Daemon Mode

For persistent background execution:

```bash
# Start the scheduler daemon
gitty scheduler start

# Check status
gitty scheduler status

# Stop the daemon
gitty scheduler stop
```

**Daemon Behavior:**

- Forks/detaches from terminal
- Creates PID file for tracking
- Writes logs to file
- Survives terminal session end
- Single instance enforced via PID file

**Platform Differences:**

| Platform | Mechanism |
|----------|-----------|
| Unix | `daemonize` crate (double-fork) |
| Windows | `DETACHED_PROCESS` creation flag |

## CLI Commands

### Start

```bash
gitty scheduler start
```

Starts the background scheduler daemon. Creates PID file and detaches from terminal.

### Stop

```bash
gitty scheduler stop
```

Stops the background scheduler daemon by reading PID file and terminating process.

### Status

```bash
gitty scheduler status
```

Shows current scheduler state:

```
Scheduler Status
================
Enabled: true
Running: true (PID: 12345)
Last run: 2024-01-15 09:30:00
Next run: 2024-01-15 10:30:00
Macro: __scheduler_default
Trigger: Simple (every 60 minutes)
Power policy: RunAlways
```

### Set Configuration

```bash
# Enable/disable
gitty scheduler set --enabled true
gitty scheduler set --enabled false

# Set interval
gitty scheduler set --interval 30

# Set macro
gitty scheduler set --macro-id <uuid>

# Set power policy
gitty scheduler set --power-policy AcOnly
```

## GUI Configuration

Navigate to **Settings** → **Scheduler**:

- **Enable scheduler** — Master toggle
- **Interval** — Minutes between checks
- **Time window** — Optional start/end times
- **Days** — Which days to run (all / weekdays / custom)
- **Power policy** — Battery-aware behavior
- **Macro selection** — Which macro to run

Changes apply immediately (next tick uses new config).

## Scheduler & Health

After each scheduled macro execution:

1. Health is re-evaluated
2. Notifications are generated (if configured)
3. Activity log is updated

This ensures the dashboard reflects the latest state without manual intervention.

## Scheduler & Liveness

The scheduler also runs liveness probes on its tick:

```
Tick:
  1. Check if macro should run → Execute if yes
  2. Check liveness probes → Probe if interval elapsed
  3. Update health
  4. Generate notifications
  5. Sleep
```

Each environment tracks its own `last_probe_timestamp` independently.

## Best Practices

### Interval Selection

| Use Case | Recommended Interval |
|----------|---------------------|
| Keep remotes current | 30-60 minutes |
| Active development | 15-30 minutes |
| Occasional contributor | 2-4 hours |
| CI/CD monitoring | 5-10 minutes |

### Time Windows

Use time windows to:
- Avoid running during off-hours (waste of resources)
- Concentrate activity during work hours
- Respect "quiet hours" for notifications

Example (business hours only):
```json
{
  "time_window": { "start": "09:00", "end": "18:00" },
  "days": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
}
```

### Power Policy

- **Desktops**: `RunAlways` (no battery)
- **Laptops (always plugged)**: `RunAlways`
- **Laptops (mobile)**: `AcOnly` or `BatteryThreshold: 30`

### Macro Selection

- **Default (`fetch`)**: Safe for all workflows
- **`pull`**: Destructive if you have uncommitted work — use with conditions
- **Custom**: Test thoroughly before scheduling

## Troubleshooting

### Scheduler not running

**GUI:**
1. Check that scheduler is enabled in Settings
2. Verify the GUI hasn't been suspended (macOS app nap)
3. Check logs for errors

**CLI:**
1. Check status: `gitty scheduler status`
2. Verify PID file exists and process is running
3. Check logs for errors

### Scheduler runs but macro fails

1. Test the macro manually: `gitty macro run <name>`
2. Check that the macro works on all target repositories
3. Review per-repo job output for specific failures

### Wrong next_run time

The `next_run` prediction is based on last_run + interval. It may be inaccurate if:
- Time window constraints push actual run later
- Power policy skips an execution
- Manual execution occurred outside schedule

### High CPU/battery usage

1. Increase interval (check less frequently)
2. Use time windows to limit active periods
3. Set `AcOnly` power policy
4. Check that macros aren't doing heavy work

### Daemon won't start

1. Check for existing PID file (stale lock)
2. Verify permissions on config directory
3. Check logs for startup errors
4. Try foreground mode first to see errors

## Logs

Scheduler logs to:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\logs\scheduler.log` |
| macOS | `~/Library/Application Support/gitty/logs/scheduler.log` |
| Linux | `~/.config/gitty/logs/scheduler.log` |

Log entries include timestamps, trigger evaluations, macro execution start/end, and errors.

## See Also

- [Macros](macros.md) — Automated operation sequences
- [Health](health.md) — Post-schedule health evaluation
- [Notifications](domain.md#notification) — Alert configuration