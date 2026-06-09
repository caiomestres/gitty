# Changes View

The **Changes View** answers "What changed across my workspace?" — a unified timeline of commits, authors, and activity across all your repositories.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Changes                                           [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Time Window: [Last 24 hours ▼]  Group by: [Author ▼]│ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  Caio Mestres (3 repositories, 12 commits)                 │
│  ═══════════════════════════════════════════════════════  │
│                                                             │
│  📁 myapp                                                  │
│    • a1b2c3d  feat: add user authentication      2h ago   │
│    • b2c3d4e  fix: handle edge case in validation  5h ago │
│                                                             │
│  📁 api                                                    │
│    • c3d4e5f  refactor: extract middleware         3h ago │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  Jane Smith (2 repositories, 5 commits)                  │
│  ═══════════════════════════════════════════════════════  │
│                                                             │
│  📁 web                                                    │
│    • d4e5f6g  style: update button colors          1h ago │
│                                                             │
│  [1] [2] ... [5]                        Show [25 ▼]         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Time Windows

Select the time period for change display:

| Window | Description |
|--------|-------------|
| **Last 24 hours** | Since this time yesterday |
| **Last 7 days** | Past week |
| **Last 30 days** | Past month |
| **Custom** — Pick dates | Specific date range |

### Time Zone

All times displayed in **local system time**.

## Grouping Options

Changes can be grouped by different dimensions:

### By Author

Groups commits by commit author:

```
Caio Mestres (5 commits across 2 repos)
├─ myapp (3)
│   ├─ a1b2c3d feat: add auth
│   └─ b2c3d4e fix: validation
└─ api (2)
    ├─ c3d4e5f refactor: middleware
    └─ ...

Jane Smith (3 commits across 1 repo)
└─ web (3)
    ├─ d4e5f6g style: buttons
    └─ ...
```

**Best for:** Standup preparation, team activity review

### By Repository

Groups commits by repository:

```
myapp (5 commits by 2 authors)
├─ a1b2c3d Caio — feat: add auth
├─ b2c3d4e Caio — fix: validation
└─ e5f6g7h Jane — docs: README

api (3 commits by 1 author)
├─ c3d4e5f Caio — refactor: middleware
└─ ...
```

**Best for:** Repository-focused review, identifying stale repos

### By Branch

Groups commits by branch:

```
main (8 commits by 3 authors)
├─ myapp: a1b2c3d Caio — feat: add auth
├─ api: c3d4e5f Caio — refactor: middleware
└─ web: d4e5f6g Jane — style: buttons

feature/login (2 commits by 1 author)
├─ myapp: x9y8z7w Caio — wip: login flow
└─ myapp: w7z8y9x Caio — wip: oauth
```

**Best for:** Understanding branch activity, release preparation

### Flat (No Grouping)

Simple chronological list:

```
• a1b2c3d Caio — myapp — feat: add auth (2h ago)
• d4e5f6g Jane — web — style: buttons (2h ago)
• c3d4e5f Caio — api — refactor: middleware (3h ago)
• b2c3d4e Caio — myapp — fix: validation (5h ago)
```

**Best for:** Full chronological timeline, quick scanning

## Commit Display

### Commit Entry

```
• a1b2c3d  feat: add user authentication      2h ago   [📋]
  └─ Caio Mestres — myapp (main)
```

**Elements:**

| Element | Description |
|---------|-------------|
| **Dot** | Visual marker |
| **Hash** | Short SHA (7 chars) — click to copy full |
| **Message** | First line of commit message |
| **Time** | Relative (hover for absolute timestamp) |
| **Copy** | Copy full hash button |

**Hover reveals:**
- Full timestamp
- Complete commit message (if multi-line)
- Author email (if available)

### Per-Repository "Show All Branches"

By default, changes view shows only the default branch (usually `main` or `master`).

Toggle icon to show all branches:

```
📁 myapp  [🌲 Show all branches]
```

When expanded:

```
📁 myapp
├─ main (3 commits)
│   • a1b2c3d feat: add auth
│   • b2c3d4e fix: validation
│
└─ feature/login (2 commits)
    • x9y8z7w wip: login flow
    • w7z8y9x wip: oauth
```

## Repository Headers

Repository grouping shows summary:

```
📁 myapp  🟢 clean  2↑ 0↓  [Open in GitHub]
```

| Element | Meaning |
|---------|---------|
| **📁 Icon** | Folder indicator |
| **Name** | Repository name (click for detail) |
| **Status** | 🟢 clean / 🟡 dirty / 🔴 missing |
| **Arrows** | Ahead/behind remote |
| **Link** | Open remote URL (if configured) |

## Author Information

Author groupings show:

```
Caio Mestres <caio@example.com>
3 repositories, 12 commits, last active 2 hours ago
```

- Name from Git config
- Email (truncated for privacy in shared screenshots)
- Repository count
- Total commits in window
- Last activity time

## Empty States

### No Repositories

```
No repositories to show changes for.

Add a scan root and scan for repositories to see changes.
[Go to Settings]
```

### No Changes in Window

```
No commits found in the last 24 hours.

Try extending the time window or check that repositories
have recent activity.
[Show last 7 days]
```

### All Repositories Missing

```
All repositories are showing as "Missing."

Paths may have changed. Try rescanning or updating scan roots.
[Rescan All]
```

## Pagination

Changes are paginated by individual entries, not by groups:

```
Showing 1-25 of 47 changes    [1] [2] [3] [Next]
```

**Important:** Group headers may repeat across page breaks. A group that starts on page 1 may continue on page 2.

## Filtering

### Text Search

Filter commits by:
- Commit message content
- Author name
- Repository name
- Hash (full or partial)

Real-time filtering as you type.

### Repository Filter

Limit to specific repositories:

```
Show: [All repositories ▼]
     ├─ All repositories
     ├─ Work group
     ├─ Personal group
     ├─ ────────────────
     ├─ myapp ★
     ├─ api
     └─ web
```

### Author Filter

Limit to specific authors:

```
Authors: [All ▼]
       ├─ All
       ├─ Caio Mestres (12)
       ├─ Jane Smith (5)
       └─ John Doe (3)
```

## Use Cases

### Daily Standup

1. Set window to "Last 24 hours"
2. Group by Author
3. Review what team members worked on
4. Identify blockers (repos with unmerged work)

### Weekly Review

1. Set window to "Last 7 days"
2. Group by Repository
3. Identify which repos are active vs. stale
4. Plan next week's focus

### Release Preparation

1. Set window to "Last 30 days"
2. Group by Branch
3. Review all branches that need merging
4. Identify release notes candidates

### Personal Activity

1. Filter to your author
2. Set window to "Last 7 days"
3. Review your own progress
4. Find that commit you forgot about

## Performance

### Lazy Loading

- Commits fetched on-demand as you scroll
- First page loads immediately
- Subsequent pages load in background

### Caching

- Change data is not cached (always fresh)
- Re-fetched on each view visit
- Uses existing Git metadata (fast)

### Large Workspaces

For workspaces with many repositories:

1. Use shorter time windows (faster)
2. Filter to specific repositories
3. Pagination keeps initial load responsive

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1-4` | Time window (24h / 7d / 30d / custom) |
| `G` | Cycle grouping (Author/Repo/Branch/Flat) |
| `F` | Focus filter search |
| `R` | Refresh / reload commits |
| `←/→` | Previous/next page |

## Known Limitations

1. **Local commits only** — Changes not yet pushed won't appear
2. **Default branch first** — Other branches require manual toggle
3. **No diff viewing** — Commit messages only (diff in repo detail)
4. **Author matching** — Based on Git commit author, not Gitty user

## See Also

- [Repository Detail](repository.md) — View individual repository commits
- [Dashboard](dashboard.md) — Quick activity overview
- [Activity Log](activity.md) — All workspace operations (not just commits)