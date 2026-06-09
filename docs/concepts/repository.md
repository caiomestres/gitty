# Repository

The **Repository** is the fundamental unit Gitty operates on. This page covers repository identity, re-linking, and lifecycle.

## Repository Identity

### UUID Assignment

When Gitty discovers a `.git` directory, it assigns a stable UUID based on the repository's root commit hash. This ensures:

- **Identity survives moves** — Relocating a repository preserves its UUID
- **Identity survives renames** — Changing the directory name doesn't affect UUID
- **Cross-machine portability** — Same repository on different machines has the same UUID

### Root Commit Fingerprint

The UUID is derived from the root commit's hash, which is immutable for the repository's lifetime.

```
UUID = hash(root_commit_hash + "gitty")
```

This deterministic approach enables **re-linking** when a repository is moved.

## Repository State

### Active

The repository's path exists and is accessible. Gitty can perform operations and read status.

### Missing

The repository's path no longer exists or is inaccessible. This occurs when:
- Directory was deleted
- Directory was moved (without Gitty's knowledge)
- External drive is disconnected
- Permissions changed

Missing repositories remain in the registry but are excluded from:
- Health score calculation
- Bulk operations
- Scheduler runs

### Re-linking

When scanning discovers a repository at a new path, Gitty attempts to match it against missing repositories by root commit fingerprint.

**Matching Rules:**
- **Unambiguous match** — Exactly one missing repo with same fingerprint → automatic re-link
- **Ambiguous match** — Multiple missing repos with same fingerprint → manual resolution required
- **No match** — New repository registered with new UUID

**Manual Re-linking:**

If automatic re-linking fails, you can:
1. Remove the old (missing) entry and re-scan
2. Manually edit config.json (not recommended)

## Repository Information

### Stored Metadata

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "path": "/home/user/projects/myapp",
  "name": "myapp",
  "group_id": "work",
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
```

### Dynamic Information

Gitty reads current Git state on demand:

- Current branch or detached HEAD state
- Dirty status (uncommitted changes)
- Ahead/behind remote
- Last commit information
- Remote tracking configuration

This information is not stored in Config — it's read fresh from Git each time.

## Repository Operations

### Discovery

Repositories are discovered by scanning **Scan Roots**:

```bash
# Scan a directory
gitty scan ~/projects
```

The scan:
1. Walks the directory tree recursively
2. Identifies `.git` directories
3. Calculates root commit fingerprint
4. Assigns or re-uses UUID
5. Registers in Config

### Removal

You can remove a repository from Gitty without deleting files:

```bash
# Unregister a repository
gitty unregister <repo-uuid>
```

This removes Gitty's tracking but leaves the `.git` directory untouched.

### Group Assignment

Move repositories between groups:

```bash
# Assign to a group
gitty group assign <repo-uuid> <group-name>
```

### Tag Management

Add or remove tags:

```bash
# Add a tag
gitty tag add <repo-uuid> favorite

# Remove a tag
gitty tag remove <repo-uuid> favorite
```

## Repository Detail View

The GUI provides a detailed view for each repository:

- **Branch & Status** — Current branch, dirty indicator, ahead/behind
- **Remotes** — Configured remotes and their URLs
- **Recent Commits** — Last 10 commits with messages
- **Tags** — Assigned tags with inline editing
- **Group** — Current group with dropdown to change
- **Health** — Current health check results
- **Liveness** — Endpoint status (if configured)

## Best Practices

### Organizing Repositories

1. **Use Groups for structure** — Mirror your mental model (`work/frontend`, `personal/dotfiles`)
2. **Use Tags for status** — Mark active projects with `favorite`, flag issues with `needs-review`
3. **Keep Scan Roots focused** — Avoid scanning your entire home directory; use specific project directories

### Handling Moves

- **Preferred**: Move within a Scan Root, then re-scan
- **Acceptable**: Remove old entry, add new Scan Root at new location
- **Avoid**: Multiple copies of the same repository (creates ambiguity)

### Performance

- Scans are fast (typically <5 seconds for hundreds of repos)
- Status reads are cached briefly in the GUI
- Fetch/pull operations are parallelized where safe

## Troubleshooting

### Repository shows as Missing

1. Check if the path still exists
2. If moved, re-scan the new location
3. If deleted, consider `gitty unregister` to clean up

### Duplicate entries after move

This happens when Gitty couldn't automatically re-link. Resolve by:
1. Identify which entry is the old (missing) one
2. `gitty unregister <old-uuid>`
3. Re-scan if needed

### Wrong repository linked

Very rare — requires identical root commits (forked repos). Manually unregister the incorrect entry and re-scan.

## See Also

- [Groups & Tags](organization.md) — Organization strategies
- [Health](health.md) — Monitoring repository health
- [CLI Reference](../cli/core.md) — Repository commands