# Organization Commands

Organization commands manage groups, tags, and repository filtering.

## group

Manage hierarchical groups.

### group list

List all groups.

```bash
gitty group list [OPTIONS]
```

**Examples:**

```bash
# List groups
gitty group list

# As tree
gitty group list --tree

# With counts
gitty group list --with-counts
```

**Output (flat):**

```
GROUP                  REPOSITORIES
──────────────────────────────────
Ungrouped              3
work                   8
work/backend           4
work/frontend          4
personal               2
```

**Output (tree):**

```
Ungrouped (3 repos)
work (8 repos)
├── backend (4 repos)
└── frontend (4 repos)
personal (2 repos)
```

**Options:**

| Option | Description |
|--------|-------------|
| `--tree` | Show hierarchical tree |
| `--with-counts` | Include repository counts |
| `--format` | Output format |

### group create

Create a new group.

```bash
gitty group create [OPTIONS] <NAME>
```

**Examples:**

```bash
# Create root-level group
gitty group create work

# Create nested group
gitty group create work/backend

# With explicit parent
gitty group create backend --parent work
```

**Behavior:**

- Creates group at specified path
- Creates parent groups if needed (with `--create-parents`)
- Fails if group already exists at that level

**Output:**

```
Created group 'work/backend'
```

**Options:**

| Option | Description |
|--------|-------------|
| `--parent <name>` | Explicit parent group |
| `--create-parents` | Create parent groups if missing |

**Arguments:**

| Argument | Description |
|----------|-------------|
| `NAME` | Group name (use `/` for nesting) |

### group rename

Rename a group.

```bash
gitty group rename <OLD_NAME> <NEW_NAME>
```

**Examples:**

```bash
# Rename group
gitty group rename work/backend work/services
```

**Output:**

```
Renamed 'work/backend' to 'work/services'
4 repositories updated
```

### group delete

Delete a group.

```bash
gitty group delete [OPTIONS] <NAME>
```

**Examples:**

```bash
# Delete group (move repos to Ungrouped)
gitty group delete work/legacy

# Force without confirmation
gitty group delete work/legacy --yes
```

**Warning:** Repositories in the deleted group are moved to "Ungrouped". Subgroups are deleted recursively.

**Output:**

```
Deleted group 'work/legacy'
3 repositories moved to 'Ungrouped'
1 subgroup deleted
```

**Options:**

| Option | Description |
|--------|-------------|
| `--yes`, `-y` | Skip confirmation |

### group move

Move (reparent) a group.

```bash
gitty group move <GROUP> <NEW_PARENT>
```

**Examples:**

```bash
# Move backend under services
gitty group move work/backend work/services
```

**Constraints:**
- Cannot move into own descendant (cycle prevention)
- Cannot move to same parent
- Repositories and subgroups move with group

**Output:**

```
Moved 'work/backend' to 'work/services'
New path: 'work/services/backend'
```

### group assign

Assign a repository to a group.

```bash
gitty group assign [OPTIONS] <REPO> <GROUP>
```

**Examples:**

```bash
# Assign by name
gitty group assign myapp work/mobile

# Assign by UUID
gitty group assign 550e8400-e29b-41d4-a716-446655440000 work/mobile
```

**Output:**

```
Moved 'myapp' from 'Ungrouped' to 'work/mobile'
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `REPO` | Repository name, path, or UUID |
| `GROUP` | Target group name |

### group tree

Display group tree.

```bash
gitty group tree [OPTIONS]
```

**Examples:**

```bash
# Show tree
gitty group tree

# With repositories
gitty group tree --show-repos
```

**Output:**

```
Workspace
├── Ungrouped
│   ├── temp-project
│   └── old-thing
├── work
│   ├── backend
│   │   ├── api
│   │   └── workers
│   └── frontend
│       └── web
└── personal
    ├── dotfiles
    └── blog
```

**Options:**

| Option | Description |
|--------|-------------|
| `--show-repos` | Include repositories in tree |
| `--group <name>` | Show subtree only |

## tag

Manage tags.

### tag list

List all tags in use.

```bash
gitty tag list [OPTIONS]
```

**Examples:**

```bash
# List all tags
gitty tag list

# With counts
gitty tag list --with-counts
```

**Output:**

```
TAG           REPOSITORIES
──────────────────────────
favorite      5
active        3
needs-review  2
archived      1
```

### tag add

Add a tag to a repository.

```bash
gitty tag add <REPO> <TAG>
```

**Examples:**

```bash
# Add favorite tag
gitty tag add myapp favorite

# Add custom tag
gitty tag add api needs-review
```

**Output:**

```
Added tag 'favorite' to 'myapp'
```

### tag remove

Remove a tag from a repository.

```bash
gitty tag remove <REPO> <TAG>
```

**Examples:**

```bash
# Remove tag
gitty tag remove myapp favorite
```

**Output:**

```
Removed tag 'favorite' from 'myapp'
```

## filter

Filter repositories by group or tag.

```bash
gitty filter [OPTIONS]
```

**Examples:**

```bash
# Filter by group
gitty filter --group work

# Filter by tag
gitty filter --tag favorite

# Combined
gitty filter --group work --tag active

# With format
gitty filter --group work --format json
```

**Output:**

```
REPOSITORY    GROUP    STATUS   BRANCH
──────────────────────────────────────
myapp         work     clean    main
api           work     dirty    develop
```

**Options:**

| Option | Description |
|--------|-------------|
| `--group <name>` | Filter by group name |
| `--tag <name>` | Filter by tag name |
| `--format` | Output format |

**Multiple filters:** AND logic (both conditions must match)

## Common Patterns

### Organize New Repositories

```bash
# After scanning, organize ungrouped repos
for repo in $(gitty list --group Ungrouped --format tsv | tail -n +2 | cut -f1); do
    echo "Move $repo to which group?"
    # Interactive selection...
done
```

### Tag Active Projects

```bash
# Tag repositories you've worked on recently
gitty tag add myapp active
gitty tag add api active
```

### Batch Operations by Tag

```bash
# Fetch all favorites
gitty filter --tag favorite | xargs -I {} gitty fetch {}
```

### Group Management

```bash
# Reorganize groups
# 1. Create new structure
gitty group create work/services

# 2. Move existing group
gitty group move work/backend work/services

# 3. Update assignments
gitty group assign api work/services/backend
```

## See Also

- [Groups & Tags Concepts](../concepts/organization.md) — Theory and best practices
- [Core Commands](core.md) — Repository operations
- [Automation Commands](automation.md) — Using filters in macros