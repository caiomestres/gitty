# Core Commands

Core commands manage repository discovery, status, and basic Git operations across your workspace.

## scan

Discover and register repositories in a directory.

```bash
gitty scan <PATH>
```

**Example:**

```bash
# Scan a directory
gitty scan ~/projects

# Scan with verbose output
gitty scan ~/projects --verbose
```

**Behavior:**

1. Recursively walks directory for `.git` folders
2. Calculates root commit fingerprint for each
3. Assigns UUIDs (or re-links to existing)
4. Registers in Config
5. Reports summary

**Output:**

```
Scanning: /home/user/projects
Found 42 Git repositories
- 3 new repositories registered
- 1 repository re-linked (moved)
- 38 repositories already known

Complete. Run 'gitty list' to see all repositories.
```

**Options:**

| Option | Description |
|--------|-------------|
| `--verbose`, `-v` | Show each repository found |
| `--dry-run` | Show what would be registered without doing it |

## list

List all registered repositories.

```bash
gitty list [OPTIONS]
```

**Examples:**

```bash
# Basic list
gitty list

# With group information
gitty list --show-group

# Filter by group
gitty list --group work

# JSON for scripting
gitty list --format json
```

**Output (human):**

```
REPOSITORY          GROUP         STATUS    BRANCH
───────────────────────────────────────────────────
myapp               work/mobile   clean     main
api                 work/backend  dirty     develop
web                 work/frontend clean     main
docs                personal      clean     main
old-project         Ungrouped     missing   -
```

**Columns:**

| Column | Description |
|--------|-------------|
| `REPOSITORY` | Repository name (directory) |
| `GROUP` | Assigned group or "Ungrouped" |
| `STATUS` | clean/dirty/missing |
| `BRANCH` | Current branch or "-" if missing |

**Options:**

| Option | Description |
|--------|-------------|
| `--format` | Output format: `human`, `json`, `tsv` |
| `--show-group` | Include group column |
| `--show-tags` | Include tags column |
| `--show-health` | Include health status |
| `--group <name>` | Filter to group |
| `--tag <name>` | Filter to tag |

## status

Show detailed Git status for repositories.

```bash
gitty status [OPTIONS] [REPO]
```

**Examples:**

```bash
# Status for all repositories
gitty status

# Status for specific repository
gitty status myapp

# Status with remote info
gitty status --remote
```

**Output:**

```
myapp
  Branch: main
  Status: clean
  Remote: origin/main (2 commits ahead, 0 behind)
  Last commit: 2 hours ago

api
  Branch: develop
  Status: dirty
  Modified: 3 files
  Untracked: 1 file
  Remote: origin/develop (in sync)
  Last commit: 5 minutes ago
```

**Options:**

| Option | Description |
|--------|-------------|
| `--remote` | Show remote tracking info |
| `--porcelain` | Machine-parseable format |
| REPO | Repository name, path, or UUID (optional) |

## fetch

Fetch from all remotes for repositories.

```bash
gitty fetch [REPO]
```

**Examples:**

```bash
# Fetch all repositories
gitty fetch

# Fetch specific repository
gitty fetch myapp

# Fetch with progress
gitty fetch --progress
```

**Behavior:**

- Runs `git fetch --all` for each repository
- Parallel execution where safe
- Per-repository locking prevents conflicts
- Skips "Missing" repositories

**Output:**

```
Fetching 15 repositories...
✓ myapp     (origin: 2 new commits)
✓ api       (origin: up to date)
✓ web       (origin: 5 new commits)
...

Done. 14 succeeded, 1 skipped (missing).
```

**Options:**

| Option | Description |
|--------|-------------|
| `--progress` | Show progress bar |
| `--quiet`, `-q` | Suppress output (exit code only) |
| REPO | Repository to target (default: all) |

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0` | All succeeded |
| `1` | One or more failed |

## pull

Pull updates for repositories.

```bash
gitty pull [REPO]
```

**Examples:**

```bash
# Pull all repositories
gitty pull

# Pull specific repository
gitty pull api

# Pull with rebase
gitty pull --rebase
```

**Warning:** Pull can cause merge conflicts. Use with care on repositories with uncommitted changes.

**Behavior:**

- Runs `git pull` for each repository
- Stops on first failure (unless `--continue-on-error`)
- Skips repositories with conflicts

**Output:**

```
Pulling 15 repositories...
✓ myapp     (Already up to date)
✓ api       (Merge made by the 'recursive' strategy)
✓ web       (Already up to date)
✗ utils     (error: Your local changes would be overwritten)

Done. 13 succeeded, 1 failed, 1 skipped.
```

**Options:**

| Option | Description |
|--------|-------------|
| `--rebase` | Use rebase instead of merge |
| `--continue-on-error` | Continue after failures |
| `--autostash` | Stash before pull, pop after |
| REPO | Repository to target (default: all) |

## checkout

Checkout a branch in repositories.

```bash
gitty checkout [OPTIONS] <BRANCH>
```

**Examples:**

```bash
# Checkout main in all repos
gitty checkout main

# Checkout in specific repo
gitty checkout develop --repo api

# Checkout new branch
gitty checkout -b feature/login
```

**Behavior:**

- Runs `git checkout <branch>` for each repository
- Creates branch if `-b` specified
- Skips repos where branch doesn't exist (unless `-b`)

**Output:**

```
Checking out 'main' in 15 repositories...
✓ myapp     (Switched to branch 'main')
✓ api       (Switched to branch 'main')
✗ web       (error: pathspec 'main' did not match)
...
```

**Options:**

| Option | Description |
|--------|-------------|
| `-b` | Create branch if it doesn't exist |
| `--repo <name>` | Target specific repository |
| `--continue-on-error` | Continue after failures |

**Arguments:**

| Argument | Description |
|----------|-------------|
| `BRANCH` | Branch name to checkout (required) |

## unregister

Remove a repository from Gitty tracking.

```bash
gitty unregister <REPO>
```

**Examples:**

```bash
# Unregister by name
gitty unregister old-project

# Unregister by UUID
gitty unregister 550e8400-e29b-41d4-a716-446655440000
```

**Warning:** This removes Gitty's tracking only. The actual Git repository and files are **not** deleted.

**Output:**

```
Unregistering 'old-project'...
Repository removed from Gitty.
Files preserved at: /home/user/projects/old-project
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `REPO` | Repository name, path, or UUID (required) |

## Common Patterns

### Daily Sync

```bash
# Morning routine
gitty fetch && gitty status
```

### Before Starting Work

```bash
# Ensure up to date
gitty fetch
gitty status --remote

# Pull specific project
gitty pull myapp
```

### Switching Branches

```bash
# Create feature branch in all project repos
gitty checkout -b feature/login --group work/project-a
```

### Cleaning Up

```bash
# Find and unregister missing repositories
gitty list | grep missing
# Then for each:
gitty unregister <name>
```

## Error Handling

Most commands support `--continue-on-error`:

```bash
# Pull everything, don't stop on first failure
gitty pull --continue-on-error
```

Check exit codes in scripts:

```bash
#!/bin/bash
gitty fetch
if [ $? -ne 0 ]; then
    echo "Fetch failed for some repositories"
    # Handle error
fi
```

## See Also

- [Full CLI Reference](../cli-reference.md) — Complete command list
- [Organization Commands](organization.md) — Groups and tags
- [Automation Commands](automation.md) — Macros and scheduler