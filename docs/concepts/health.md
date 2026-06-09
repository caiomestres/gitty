# Health

The Health system monitors your repositories for issues that need attention. It provides both individual repository checks and an aggregate Workspace Health score.

## Health Checks

Gitty includes four built-in health checks, each evaluating a specific criterion:

### Freshness

Detects repositories that haven't been synced with remotes recently.

**What it checks:**
- Time since last `fetch` or `pull`
- Commit age of last upstream sync

**Configuration:**
```json
{
  "health": {
    "freshness_days_warning": 3,
    "freshness_days_critical": 7
  }
}
```

| Status | Condition |
|--------|-----------|
| Healthy | Last fetch within 3 days |
| Warning | Last fetch 3-7 days ago |
| Critical | Last fetch more than 7 days ago |

**Typical causes:**
- Repository is stale/abandoned
- Scheduler not running
- Remote no longer accessible

### Divergence

Detects repositories with unpushed commits or unmerged remote changes.

**What it checks:**
- Commits ahead of remote (unpushed work)
- Commits behind remote (unfetched changes)

**Configuration:**
```json
{
  "health": {
    "divergence_ahead_warning": 5,
    "divergence_ahead_critical": 20,
    "divergence_behind_warning": 10,
    "divergence_behind_critical": 50
  }
}
```

| Status | Condition |
|--------|-----------|
| Healthy | 0-4 ahead, 0-9 behind |
| Warning | 5-19 ahead or 10-49 behind |
| Critical | 20+ ahead or 50+ behind |

**Typical causes:**
- Forgot to push feature branch
- Remote has new commits you need to pull
- Long-running branch diverged from main

### Dirty Tree

Detects repositories with uncommitted changes.

**What it checks:**
- Modified files (staged or unstaged)
- Untracked files
- Merge conflicts

**Configuration:**
```json
{
  "health": {
    "dirty_is_warning": true
  }
}
```

| Status | Condition |
|--------|-----------|
| Healthy | Working tree clean |
| Warning | Uncommitted changes exist |
| Critical | (Never — dirty is always warning) |

**Typical causes:**
- Work in progress not committed
- Debug/temporary files present
- Merge conflicts unresolved

### Detached HEAD

Detects repositories not on a branch.

**What it checks:**
- `HEAD` is detached (not pointing to a branch)
- Common after checking out a specific commit or tag

**Configuration:**
```json
{
  "health": {
    "detached_is_warning": true
  }
}
```

| Status | Condition |
|--------|-----------|
| Healthy | On a branch |
| Warning | Detached HEAD |
| Critical | (Never — detached is always warning) |

**Typical causes:**
- Checked out a specific commit for debugging
- CI/CD left repository in detached state
- Bisect in progress

## Health Status

Each check produces one of three statuses:

| Status | Icon | Meaning | Action Needed |
|--------|------|---------|---------------|
| **Healthy** | 🟢 | Within normal parameters | None |
| **Warning** | 🟡 | Attention recommended | Review when convenient |
| **Critical** | 🔴 | Action required | Address soon |

## Workspace Health Score

The aggregate health of your entire workspace, calculated as:

```
Score = (repos_not_critical / total_active_repos) × 100
```

- Missing repositories are **excluded** from calculation
- A score of 100% means no repositories are in critical state
- A score of 0% means all repositories are critical

### Score Interpretation

| Score | Assessment |
|-------|------------|
| 90-100% | Excellent — workspace is healthy |
| 70-89% | Good — some attention needed |
| 50-69% | Fair — significant issues to address |
| 0-49% | Poor — many critical issues |

### Dashboard Display

The Dashboard prominently displays:

- **Score** — Large percentage
- **Trend** — Up/down arrow comparing to last evaluation
- **Breakdown** — Count of healthy/warning/critical repos

## Health Evaluation

### When Health is Evaluated

Health is (re)evaluated:

1. **On demand** — User triggers evaluation (GUI button, CLI command)
2. **After fetch/pull** — Any successful remote sync
3. **After scheduler run** — Automated background execution
4. **On repository changes** — Group/tag assignment, new repo registration

### Manual Evaluation

**CLI:**

```bash
# Evaluate all repositories
gitty health

# Evaluate specific repository
gitty health --repo <repo-uuid>

# Health shows summary by default
gitty health
# Output: Workspace Health: 85% (17/20 repos healthy)
```

**GUI:**

- Navigate to **Health** page
- Click **Refresh** button
- View per-repo breakdown

## Health Caching

Health data is cached to enable fast dashboard loading:

- **Cache file:** `health.json` alongside `config.json`
- **Update triggers:** Evaluation events (see above)
- **TTL:** Not time-based; refreshed on significant events

The cache stores:
- Last evaluation timestamp
- Per-repository check results
- Aggregated score

## GUI Health View

The dedicated Health page provides:

### Summary Cards

- Workspace Health score (large percentage)
- Repository counts by status
- Last evaluation timestamp

### Repository Table

- Repository name with group/tag indicators
- Individual check results (icons)
- Overall status (color-coded)
- Quick actions (fetch, open detail)

### Filtering & Sorting

- Filter by status (healthy/warning/critical)
- Sort by name, status, or last evaluation
- Group by group (hierarchical view)

### Drill-Down

Click any repository to see:
- Full check details
- Raw check output
- Suggested actions
- Related activity log entries

## Best Practices

### Threshold Tuning

Adjust thresholds based on your workflow:

- **Active development:** Lower freshness thresholds (1-2 days)
- **Stable projects:** Higher thresholds acceptable (7-14 days)
- **Archive:** Exclude from health or use very high thresholds

### Handling Common Issues

| Issue | Check | Typical Fix |
|-------|-------|-------------|
| Stale repository | Freshness | Fetch or determine if abandoned |
| Unpushed commits | Divergence | Push branch or create PR |
| Unpulled changes | Divergence | Pull and merge/rebase |
| Uncommitted work | Dirty | Commit, stash, or discard |
| Detached HEAD | Detached | Checkout branch or create one |

### Proactive Health

1. **Enable scheduler** — Regular fetches keep freshness in check
2. **Use macros** — Batch operations for common fixes
3. **Review weekly** — Dedicate time to address warnings
4. **Archive old repos** — Remove or move to archive group

## Troubleshooting

### Health not updating

1. Check that health evaluation has run (timestamp on Health page)
2. Verify repository is not "Missing" (excluded from health)
3. Try manual refresh

### False positives

Some checks may flag acceptable states:

- **Dirty on purpose:** WIP branches, experiments
- **Detached intentionally:** Debugging specific commits
- **Divergence expected:** Long-running feature branches

Consider:
- Adjusting thresholds
- Using tags to mark intentional states (`experiment`, `long-term-branch`)
- Temporarily disabling checks for specific repos (not currently supported)

### Health check failures

If health evaluation itself fails:

1. Check repository path is accessible
2. Verify Git is functioning in that directory
3. Check Gitty logs for errors

## See Also

- [Liveness](liveness.md) — Service endpoint monitoring
- [Scheduler](scheduler.md) — Automated health evaluation
- [GUI Guide](../gui/health.md) — Health view walkthrough