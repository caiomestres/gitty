# Dashboard

The **Dashboard** is your workspace overview — the first thing you see when launching Gitty and the central hub for daily operations.

## Layout

```
┌─────────────────────────────────────────────────────┐
│  Dashboard                                   [⚙️]  │
├─────────────────────────────────────────────────────┤
│  Workspace Health: 85%    15 repos    Last sync: 2m │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │
│  │ myapp   │ │ api     │ │ docs    │ │ ...     │ │
│  │ 🟢 main │ │ 🟡 dev  │ │ 🟢 main │ │         │ │
│  │ 2↑ 1↓   │ │ dirty   │ │ clean   │ │         │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ │
│                                                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │
│  │ utils   │ │ config  │ │ web     │ │ + Add   │ │
│  │ 🔴 miss │ │ 🟢 main │ │ 🟡 5↑   │ │  more   │ │
│  │         │ │ clean   │ │ 10↓     │ │         │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ │
│                                                     │
├─────────────────────────────────────────────────────┤
│  [Fetch All]  [Refresh]  [Filter ▼]  [Pagination]   │
└─────────────────────────────────────────────────────┘
```

## Stats Cards

At the top, summary cards show workspace status:

| Card | Description |
|------|-------------|
| **Health Score** | Large percentage (0-100%) with trend arrow |
| **Repository Count** | Active / Total repositories |
| **Last Sync** | Time since last fetch/pull operation |
| **Quick Actions** | Buttons for common operations |

### Health Score

The large percentage represents overall workspace health:

- **100%** — All repositories healthy (green)
- **85-99%** — Most healthy, some warnings (yellow)
- **50-84%** — Several issues need attention (orange)
- **0-49%** — Many critical issues (red)

**Trend indicator:**
- **↑** — Improving since last evaluation
- **↓** — Declining (new issues appeared)
- **→** — Stable

## Repository Grid

Repository cards display key information at a glance:

### Card Anatomy

```
┌─────────────────┐
│  🦁 myapp      │ ← Name + optional mascot
│  ━━━━━━━━━━━━━  │
│  🟢 main       │ ← Current branch + status
│  2↑ 1↓         │ ← Ahead/behind remote
│  [dev][local]  │ ← Tags (if space)
│                 │
│  🟢 ●          │ ← Health + Liveness dots
└─────────────────┘
```

### Status Display

| Element | Meaning |
|---------|---------|
| **Branch** | Current branch name or "detached" |
| **Color dot** | 🟢 clean, 🟡 dirty, 🔴 missing |
| **Arrows** | ↑ commits ahead, ↓ commits behind |
| **Tags** | Assigned tags (truncated if many) |
| **Health dot** | 🟢 healthy, 🟡 warning, 🔴 critical |
| **Liveness dot** | 🟢 up, 🔴 down, ⚪ unknown |

### Card Interactions

| Action | Result |
|--------|--------|
| **Click** | Open repository detail page |
| **Right-click** | Context menu (fetch, pull, favorite) |
| **Ctrl+Click** | Multi-select (for bulk operations) |

### Grid Layout

- **Responsive** — Cards resize to fit window
- **Sortable** — By name, status, health, last active
- **Filterable** — By group, tag, health status, text search

## Quick Actions

### Fetch All

Triggers `git fetch --all` across all repositories:

1. Click **Fetch All** button
2. Confirm dialog appears (optional, based on settings)
3. Progress indicator shows per-repo status
4. Results summary when complete

### Refresh

Manually refresh the repository grid:

- Re-reads Git status for displayed repos
- Updates health indicators
- Does **not** perform fetch (use Fetch All for that)

### Filter & Search

**Filter Dropdown:**
- **Group** — Show only repositories in selected group
- **Tag** — Show only repositories with selected tag
- **Health** — Show only healthy/warning/critical repos
- **Status** — Show only clean/dirty/missing repos

**Search:**
- Type to filter by repository name
- Searches both name and path
- Real-time filtering as you type

## Empty State

When no repositories are registered:

```
┌─────────────────────────────────────────┐
│                                         │
│           🦁                            │
│                                         │
│      Welcome to Gitty!                  │
│                                         │
│      Your workspace is empty.          │
│      Add a scan root to get started.     │
│                                         │
│      [Add Scan Root]  [Learn More]     │
│                                         │
└─────────────────────────────────────────┘
```

The onboarding card appears with:
- Mascot illustration
- Brief welcome message
- Primary CTA to add scan root
- Secondary link to documentation

## Pagination

When you have many repositories, pagination controls appear:

```
[Previous]  [1] [2] [3] ... [10]  [Next]    Show [25 ▼] per page
```

**Options:**
- Items per page: 10, 25, 50, 100
- Page navigation
- Jump to first/last

## First-Run Experience

On first launch with empty workspace:

1. **Welcome modal** — Brief product introduction
2. **Scan root prompt** — Guide to add first directory
3. **First scan** — Automatically run
4. **Dashboard populated** — Show discovered repositories

## Best Practices

### Daily Workflow

1. **Morning**: Check health score, review warnings
2. **Start work**: Fetch All to get latest changes
3. **During day**: Click through to repository details as needed
4. **End of day**: Review any new critical issues

### Organization

- **Use filters** — Narrow to active projects with "favorite" tag
- **Sort by health** — Bring critical repos to top
- **Monitor trends** — Watch health score over time

### Performance

- Pagination keeps initial load fast
- Fetch All runs in parallel where safe
- Status is cached and refreshed on demand

## Troubleshooting

### Health score not updating

1. Check that health evaluation has run
2. Navigate to Health page and click Refresh
3. Verify repositories are not "Missing"

### Repository showing outdated status

1. Click Refresh button
2. Or perform Fetch All to update remote information

### Missing repositories in grid

1. Check filters (may be filtered out)
2. Check pagination (may be on another page)
3. Verify repositories are not "Missing" in Settings

### Slow loading

1. Reduce items per page
2. Use filters to show fewer repositories
3. Check for large number of untracked files (affects Git status)

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Refresh |
| `F` | Fetch All (with confirmation) |
| `/` | Focus search |
| `1-9` | Jump to page number |

## See Also

- [Repository Detail](repository.md) — Deep dive into individual repos
- [Health View](health.md) — Detailed health monitoring
- [Quick Start](../intro/quickstart.md) — Getting started guide