# Repository Detail

The **Repository Detail** page provides comprehensive information about a single repository and access to all repository-specific actions.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  ← Back    myapp    [Edit]    [Favorite ☆]    [More ▼]      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌─────────────────────────────────────┐ │
│  │              │  │ Status                              │ │
│  │   🦁         │  │ ────────────────────────────────────│ │
│  │  (mascot)    │  │ Current: main                        │ │
│  │              │  │ Status: 🟡 Dirty (3 modified)         │ │
│  │  Health: 🟢  │  │ Remote: origin/main (2↑ 1↓)          │ │
│  │  Live: 🟢    │  │                                     │ │
│  │              │  │ Last commit: 2 hours ago              │ │
│  └──────────────┘  └─────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────┐  ┌─────────────────────────────────────┐ │
│  │ Group        │  │ Tags                                │ │
│  │ ───────────  │  │ ────────────────────────────────────│ │
│  │ work/mobile ▼│  │ [favorite] [active] [+ Add]         │ │
│  └──────────────┘  └─────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Recent Commits                                       │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ • a1b2c3d  feat: add user authentication   2h ago    │ │
│  │ • e5f6g7h  fix: handle null pointer        5h ago    │ │
│  │ • i9j0k1l  docs: update README             1d ago    │ │
│  │                                                      │ │
│  │ [View Full History]                                  │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Remotes                                              │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ origin  https://github.com/user/myapp.git (fetch)    │ │
│  │ origin  https://github.com/user/myapp.git (push)     │ │
│  │ upstream https://github.com/original/myapp.git     │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Liveness                                             │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ 🟢 dev        http://localhost:3000/health  (30s)    │ │
│  │ 🟢 storybook  http://localhost:6006        (120s)    │ │
│  │                                                      │ │
│  │ [+ Add Environment]                                  │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [Fetch]  [Pull]  [Checkout...]  [Open in Editor]  [Shell] │
└─────────────────────────────────────────────────────────────┘
```

## Header

The header provides navigation and quick actions:

### Navigation

- **← Back** — Return to previous view (usually Dashboard)
- **Repository name** — Large, prominent display

### Actions

- **Edit** — Rename repository (display name only, not path)
- **Favorite** — Toggle favorite tag (☆ / ★)
- **More ▼** — Additional options:
  - Unregister repository
  - Open in file manager
  - Copy path to clipboard
  - Copy UUID to clipboard

## Status Card

The primary status information:

### Current Branch

```
Current: main
├─ Tracking: origin/main
├─ Ahead: 2 commits (unpushed)
└─ Behind: 1 commit (to pull)
```

**Branch states:**
| State | Display |
|-------|---------|
| On branch | `main`, `develop`, etc. |
| Detached | `detached at a1b2c3d` |
| Just initialized | `(no commits yet)` |

### Working Tree Status

| Status | Icon | Details |
|--------|------|---------|
| Clean | 🟢 | Nothing to commit |
| Modified | 🟡 | N files modified |
| Staged | 🟡 | N files staged |
| Untracked | 🟡 | N untracked files |
| Conflicts | 🔴 | N merge conflicts |

### Remote Status

- **Ahead (↑)** — Local commits not pushed
- **Behind (↓)** — Remote commits not pulled
- **Diverged (↕)** — Both ahead and behind

### Last Commit

- Commit hash (short)
- Author name
- Relative timestamp (hover for absolute)
- Commit message (first line)

## Mascot & Indicators

The left column shows the mascot and key status indicators:

### Mascot

- Large mascot illustration (64px+ depending on layout)
- Color changes with theme (golden for Default, luminous for Dark, Brasil colors for Brasil)

### Health Indicator

- Large colored dot with label
- Click to see health check details
- Link to Health page filtered to this repo

### Liveness Indicator

- Large colored dot with label
- Shows primary environment status
- Click to see all environments

## Organization

### Group

Dropdown to change repository's group assignment:

```
Group: [work/mobile ▼]
       ├─ work
       │   ├─ backend
       │   ├─ frontend
       │   └─ mobile ← current
       ├─ personal
       │   └─ ...
       └─ Ungrouped
```

**Moving confirmation:**
> "Moving from 'work/backend' to 'work/mobile'"

### Tags

Inline tag management:

```
Tags: [favorite ★] [active] [needs-review] [+]
```

- Click tag to remove (with confirmation)
- Click **+** to add new tag
- Type to filter available tags
- Create new tag inline

## Recent Commits

Shows last 5-10 commits:

| Field | Description |
|-------|-------------|
| Hash | Short SHA (click to copy full) |
| Message | First line of commit message |
| Author | Name (not email, for privacy) |
| Time | Relative (hover for absolute) |

**[View Full History]** — Opens detailed commit log (future feature)

## Remotes

Lists configured Git remotes:

```
origin
├─ URL: https://github.com/user/myapp.git
├─ Fetch: ✓
└─ Push: ✓
```

- Name with fetch/push indicators
- Full URL (click to copy)
- Default remote highlighted

## Liveness

Configured HTTP endpoints for this repository:

| Column | Description |
|--------|-------------|
| Status | 🟢 up / 🔴 down / ⚪ unknown |
| Name | Environment name (dev, staging, etc.) |
| Endpoint | URL (truncated if long) |
| Interval | Probe frequency |

**Actions:**
- Click row to see probe history
- **[+ Add Environment]** — Configure new endpoint
- Right-click to edit/remove

### Add Environment Dialog

```
Add Environment
───────────────
Name: [dev              ]
URL:  [http://localhost:3000/health]
Interval: [60 ▼] seconds

[Cancel]  [Add]
```

## Action Bar

Bottom action buttons for common operations:

### Fetch

- `git fetch --all` for this repository only
- Shows progress spinner
- Updates remote status when complete

### Pull

- `git pull` for this repository only
- Confirmation if dirty (will attempt merge)
- Shows progress and result

### Checkout...

Opens dialog with:
- List of local branches
- List of remote branches
- Input for new branch name
- Search/filter branches

### Open in Editor

Opens repository path in:
- Default file manager
- Configured editor (VS Code, etc.) if set

### Shell

Opens terminal at repository path:
- Platform-native terminal
- Inherits environment
- Working directory set to repo root

## Error States

### Missing Repository

If the repository path no longer exists:

```
┌─────────────────────────────────────────────┐
│                                             │
│  ⚠️ Repository Path Not Found                │
│                                             │
│  Expected: /home/user/projects/myapp        │
│                                             │
│  This repository may have been moved.       │
│  Try rescanning or update the path.         │
│                                             │
│  [Locate...]  [Remove]  [Help]             │
│                                             │
└─────────────────────────────────────────────┘
```

**Options:**
- **Locate...** — Browse to new location (re-linking)
- **Remove** — Unregister from Gitty (keeps files)
- **Help** — Documentation on re-linking

### Uninitialized Repository

If the repository has no commits yet:

```
Current: (no commits yet)
Status: Empty repository

Create your first commit to see history here.
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Esc` | Go back |
| `F` | Fetch |
| `P` | Pull |
| `C` | Checkout dialog |
| `E` | Open in editor |
| `T` | Focus tag input |
| `⌘/Ctrl + C` | Copy path |

## See Also

- [Dashboard](dashboard.md) — Back to workspace overview
- [Health](health.md) — Detailed health monitoring
- [Liveness](../concepts/liveness.md) — Endpoint monitoring concept
- [Groups & Tags](../concepts/organization.md) — Organization concepts