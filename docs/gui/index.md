# GUI Guide

Gitty's desktop application provides a visual interface for all workspace operations. This guide covers each page and feature.

## Interface Overview

```
┌─────────────────────────────────────────────────────────────┐
│  🦁 Gitty    Dashboard  Health  Changes  Activity  [Search]  │ ← Top Bar
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│  DASH    │                                                  │
│  BOARD   │              Main Content Area                   │
│          │                                                  │
│  HEALTH  │         (Dashboard, Repository Detail,          │
│          │          Health View, Settings, etc.)           │
│ CHANGES  │                                                  │
│          │                                                  │
│ ACTIVITY │                                                  │
│          │                                                  │
├──────────┴──────────────────────────────────────────────────┤
│  [Theme]  [Notifications]  [Status: 15 repos]    [Scheduler] │ ← Bottom Bar
└─────────────────────────────────────────────────────────────┘
```

## Navigation Structure

### Sidebar

The sidebar provides primary navigation between major views:

| Item | Description |
|------|-------------|
| **Dashboard** | Workspace overview, repository grid |
| **Health** | Health monitoring and drill-down |
| **Changes** | What changed across workspace |
| **Activity** | Operation history and audit log |
| **Groups** | Group tree and organization |
| **Macros** | Macro management and execution |
| **Settings** | Configuration and preferences |

### Top Bar

Contains:
- **Mascot/Logo** — Click to return to Dashboard
- **Navigation tabs** — Quick access to main views
- **Global search** — Find repositories, groups, macros
- **Notifications** — Alert center with badge

### Bottom Bar

Contains:
- **Theme switcher** — Quick theme toggle
- **Status** — Repository count and summary
- **Scheduler indicator** — Running/stopped status

## Page Guide

### [Dashboard](dashboard.md)

Your workspace at a glance. Repository cards, quick actions, and workspace summary.

### [Repository Detail](repository.md)

Deep dive into a single repository. Branch info, remotes, commits, and actions.

### [Health View](health.md)

Health monitoring with traffic-light status and drill-down capabilities.

### [Changes View](changes.md)

"What changed?" across your workspace. Time-windowed commit history.

### [Activity Log](activity.md)

History of operations, macro executions, and state changes.

### [Groups](groups.md)

Manage the hierarchical group tree and repository assignments.

### [Macros](macros.md)

Create, edit, and execute macros with visual step editor.

### [Settings](settings.md)

Configure scan roots, scheduler, notifications, and preferences.

### [Themes](themes.md)

Switch between Default, Dark, and Brasil themes with live preview.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + K` | Open global search |
| `Ctrl/Cmd + 1` | Go to Dashboard |
| `Ctrl/Cmd + 2` | Go to Health |
| `Ctrl/Cmd + 3` | Go to Changes |
| `Ctrl/Cmd + 4` | Go to Activity |
| `Ctrl/Cmd + ,` | Open Settings |
| `Ctrl/Cmd + F` | Fetch all (with confirmation) |

## Status Indicators

Throughout the interface, icons and colors convey status:

### Repository Status

| Icon | Meaning |
|------|---------|
| 🟢 | Clean, up-to-date |
| 🟡 | Warning (diverged, dirty) |
| 🔴 | Critical or action needed |
| ⚪ | Missing (path not found) |
| 🔄 | Operation in progress |

### Health Status

| Icon | Status |
|------|--------|
| 🟢 | Healthy |
| 🟡 | Warning |
| 🔴 | Critical |
| ⚪ | Not evaluated |

### Liveness Status

| Icon | Status |
|------|--------|
| 🟢 | Up (reachable) |
| 🔴 | Down (unreachable) |
| ⚪ | Unknown (not probed) |

## Tooltips

Hover over any status indicator for a descriptive tooltip:

- **Repository badges** — "5 commits behind main"
- **Health dots** — "Freshness check: last fetch 3 days ago"
- **Liveness dots** — "Dev environment: responding 200 OK"

## Notifications

Notifications appear in the in-app panel and (if enabled) as OS-native toasts:

- Critical health changes
- Scheduler completions
- Macro execution completions
- Liveness endpoint failures

Click notifications to navigate to relevant views.

## Getting Help

- **Tooltips** — Hover for context
- **Empty states** — Helpful guidance when no data
- **Error messages** — Recovery suggestions included
- **This documentation** — Comprehensive reference

## See Also

- [Core Concepts](../concepts/index.md) — Understanding the domain
- [CLI Reference](../cli/index.md) — Command-line alternative
- [Quick Start](../intro/quickstart.md) — First steps