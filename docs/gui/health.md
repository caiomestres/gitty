# Health View

The **Health View** provides comprehensive monitoring of repository health with traffic-light status indicators and drill-down capabilities.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Health                                            [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Workspace Health: 85%                        [Refresh]│ │
│  │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━              │ │
│  │ ████████████████████░░░░░  17/20 healthy               │ │
│  │                                                        │ │
│  │ 🟢 Healthy: 14    🟡 Warning: 3    🔴 Critical: 0     │ │
│  │ Last evaluated: 5 minutes ago                        │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Filter: [All ▼]  Sort: [Status ▼]  Search: [______]  │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  Repository          Status    Freshness  Divergence  Dirty │
│  ─────────────────────────────────────────────────────────  │
│  🟡 myapp            warning   ⚠️ 3d       ✓          ✓    │
│  🔴 api              critical  ✓          ⚠️ 25↑       ✓    │
│  🟢 web              healthy   ✓          ✓          ✓    │
│  🟢 docs             healthy   ✓          ✓          ✓    │
│  🟡 config           warning   ✓          ✓          ⚠️    │
│                                                             │
│  [1] [2] [3] ... [10]                  Show [25 ▼]        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Health Summary Card

The top section provides workspace-level health overview:

### Health Score

- **Large percentage** (0-100%)
- **Progress bar** — Visual representation
- **Ratio** — "17/20 healthy" repositories

### Status Breakdown

| Status | Count | Meaning |
|--------|-------|---------|
| 🟢 **Healthy** | 14 | All checks passed |
| 🟡 **Warning** | 3 | Some checks warning |
| 🔴 **Critical** | 0 | At least one check critical |

**Note:** Missing repositories are excluded from these counts.

### Last Evaluation

Timestamp of when health was last evaluated:

- **Just now** — Less than 1 minute
- **X minutes ago** — Recent
- **X hours ago** — Older

**Refresh button** — Manually trigger re-evaluation

## Filtering & Sorting

### Filter Dropdown

Filter repositories by status:

- **All** — Show all repositories
- **Healthy only** — Hide warnings and critical
- **Warning and above** — Hide healthy
- **Critical only** — Show only critical (focus mode)

### Sort Options

| Sort | Description |
|------|-------------|
| **Status** — Critical first | Severity descending |
| **Status** — Healthy first | Severity ascending |
| **Name** | Alphabetical |
| **Last Evaluated** | Most recent first |
| **Group** | By group hierarchy |

### Search

Filter by repository name (real-time):

- Searches both name and path
- Case-insensitive
- Clears other filters temporarily

## Repository Table

Columns show per-repository health information:

| Column | Description |
|--------|-------------|
| **Repository** | Name with group/tag indicators |
| **Status** | Overall status icon and label |
| **Freshness** | Days since last fetch |
| **Divergence** | Ahead/behind counts |
| **Dirty** | Clean / modified indicator |
| **Detached** | On branch / detached |

### Status Icons

| Icon | Status |
|------|--------|
| 🟢 | Healthy — All checks passed |
| 🟡 | Warning — At least one check warning |
| 🔴 | Critical — At least one check critical |
| ⚪ | Not evaluated — Health data missing |

### Check Columns

Each check column shows specific results:

#### Freshness

| Icon | Status | Meaning |
|------|--------|---------|
| ✓ | Healthy | Within threshold |
| ⚠️ | Warning | Approaching threshold |
| ❌ | Critical | Exceeded threshold |
| — | N/A | Not configured |

Hover shows: "Last fetch: 3 days ago (warning at 3 days)"

#### Divergence

| Display | Meaning |
|---------|---------|
| ✓ | In sync with remote |
| ↑5 | 5 commits ahead (unpushed) |
| ↓3 | 3 commits behind (unpulled) |
| ↕15 | 15 commits diverged |

Hover shows: "15 ahead, 0 behind (critical at 20 ahead)"

#### Dirty

| Icon | Status |
|------|--------|
| ✓ | Clean working tree |
| ⚠️ | Uncommitted changes |

Hover shows: "3 modified files, 1 staged, 2 untracked"

#### Detached

| Icon | Status |
|------|--------|
| ✓ | On branch |
| ⚠️ | Detached HEAD |

Hover shows: "Detached at commit a1b2c3d"

## Row Interactions

### Click

Click any row to open the repository detail page with health tab active.

### Right-Click

Context menu:

- **Open detail** — Navigate to repository page
- **Fetch** — Fetch this repository only
- **Mark as viewed** — Acknowledge (no state change, just UI)
- **Add to favorites** — Quick tag assignment

### Expand (Future)

Click expand arrow to see inline:

- All check details
- Raw check output
- Suggested actions

## Drill-Down Detail

When viewing a specific repository's health:

```
┌─────────────────────────────────────────────────────────────┐
│  ← Back    myapp — Health Details                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Overall Status: 🟡 Warning                                 │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Freshness Check                                      │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ Status: 🟡 Warning                                   │ │
│  │                                                      │ │
│  │ Last fetch: 4 days ago                               │ │
│  │ Threshold: Warning at 3 days, Critical at 7 days   │ │
│  │                                                      │ │
│  │ [Fetch Now]                                          │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Divergence Check                                     │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ Status: 🟢 Healthy                                   │ │
│  │                                                      │ │
│  │ In sync with origin/main                             │ │
│  │ 0 commits ahead, 0 commits behind                  │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Dirty Tree Check                                     │ │
│  │ ──────────────────────────────────────────────────────│ │
│  │ Status: 🟢 Healthy                                   │ │
│  │                                                      │ │
│  │ Working tree is clean                                │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Check Cards

Each health check displayed as a card:

- **Check name** — Freshness, Divergence, Dirty, Detached
- **Status** — Large icon and label
- **Details** — Specific metrics and values
- **Thresholds** — Warning and critical levels
- **Action button** — Quick fix (Fetch Now, etc.)

### Suggested Actions

Based on check results:

| Check Status | Suggested Action |
|--------------|------------------|
| Freshness warning | "This repository hasn't been fetched in 4 days. [Fetch Now]" |
| Divergence critical | "25 commits behind remote. Consider pulling soon. [Pull Now]" |
| Dirty warning | "3 uncommitted changes. [Commit], [Stash], or [Discard]" |
| Detached warning | "Not on any branch. [Checkout main] or [Create branch]" |

## Threshold Configuration

Health thresholds are configured per-workspace in Settings:

```
Health Check Thresholds
───────────────────────
Freshness:
  Warning after:  [3  ] days
  Critical after: [7  ] days

Divergence:
  Warning (ahead):  [5  ] commits
  Critical (ahead): [20 ] commits
  Warning (behind): [10 ] commits
  Critical (behind):[50 ] commits
```

Changes apply immediately (next evaluation uses new thresholds).

## Evaluation Triggers

Health is automatically evaluated:

1. **On demand** — Click Refresh button
2. **After fetch/pull** — Any successful remote sync
3. **After scheduler run** — Background execution complete
4. **On repository changes** — New repo, group change, etc.

## Health Caching

Health data is cached for performance:

- **Storage:** `health.json` alongside `config.json`
- **Updates:** Triggered by events above
- **Not time-based:** Doesn't expire; always current

This enables fast dashboard loading without Git operations.

## Empty States

### No Repositories

```
No repositories registered.

Add a scan root in Settings to discover repositories.
[Go to Settings]
```

### Health Not Evaluated

```
Health data not available.

Click Refresh to evaluate repository health.
[Refresh]
```

### All Healthy

When all repositories are healthy (celebration):

```
🎉 All repositories are healthy!

14 repositories, 0 warnings, 0 critical issues.
```

## Best Practices

### Monitoring Routine

1. **Check daily** — Review health view each morning
2. **Address warnings** — Don't let them accumulate
3. **Fix critical promptly** — These block workflows
4. **Watch trends** — Health declining? Investigate why

### Threshold Tuning

Adjust based on your workflow:

- **Active development:** Lower freshness thresholds (1-2 days)
- **Stable projects:** Higher thresholds acceptable (7-14 days)
- **Long-running branches:** Increase divergence thresholds

### Team Workflows

- Share threshold configurations
- Document team conventions ("always pull before lunch")
- Use macros to automate common fixes

## Troubleshooting

### Health shows stale data

1. Click **Refresh** button
2. Verify evaluation completed (check timestamp)
3. Check for errors in logs

### False warnings

Some acceptable states trigger warnings:

- **Intentionally dirty:** WIP branches, experiments
- **Expected divergence:** Long-running feature branches
- **Detached for debugging:** Checking specific commits

Consider:
- Adjusting thresholds
- Using tags to mark intentional states
- Temporarily ignoring (no suppression feature yet)

### Check not running

1. Verify check is enabled in Settings
2. Ensure repository is not "Missing"
3. Check Git access to repository

### Performance issues

If health evaluation is slow:

1. Check for large repositories (many refs)
2. Verify network access to remotes
3. Consider reducing repository count per workspace

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Refresh health evaluation |
| `F` | Filter toggle |
| `S` | Sort toggle |
| `/` | Focus search |
| `Esc` | Clear filters |

## See Also

- [Health Concepts](../concepts/health.md) — How health checks work
- [Repository Detail](repository.md) — Per-repo deep dive
- [Scheduler](../concepts/scheduler.md) — Automated health updates