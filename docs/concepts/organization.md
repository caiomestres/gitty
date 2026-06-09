# Groups & Tags

Groups and Tags are the two complementary organization systems in Gitty. Understanding their differences helps you build an effective workspace structure.

## Groups: Hierarchical Structure

Groups provide a tree-based organizational structure for repositories.

### Characteristics

- **Exactly one group per repository** — A repository cannot exist in multiple groups
- **Arbitrary nesting** — Groups can contain other groups to any depth
- **Ungrouped default** — New repositories start here until assigned

### Use Cases

| Pattern | Example Structure | Best For |
|---------|-------------------|----------|
| By project | `work/project-a`, `work/project-b` | Multi-repo projects |
| By team | `backend/services`, `frontend/apps` | Large organizations |
| By client | `client-acme`, `client-globex` | Agency/consulting |
| By type | `dotfiles`, `experiments`, `forks` | Personal organization |
| Mixed | `work/active`, `personal/dotfiles` | Most real-world setups |

### The Group Tree

```
Workspace
├── Ungrouped
│   └── (newly discovered repos)
├── work
│   ├── backend
│   │   ├── api
│   │   └── workers
│   └── frontend
│       ├── web
│       └── mobile
└── personal
    ├── dotfiles
    └── experiments
```

### Managing Groups

**CLI:**

```bash
# List all groups
gitty group list

# Create a group
gitty group create work/backend

# Rename a group
gitty group rename work/backend work/services

# Delete a group (repos move to Ungrouped)
gitty group delete work/services

# Move a group (reparent)
gitty group move work/services work

# Assign repository to group
gitty group assign <repo-uuid> work/services

# View the tree
gitty group tree
```

**GUI:**

- Navigate to **Groups** for CRUD operations
- Use the sidebar tree for navigation
- Change a repository's group from its detail page

### Constraints

- **No cycles** — A group cannot be moved into its own descendant
- **Unique names at level** — Sibling groups cannot share names
- **Case-sensitive** — `Work` and `work` are different groups

## Tags: Cross-Cutting Labels

Tags provide flat, additive labeling for repositories.

### Characteristics

- **Zero or more per repository** — A repository can have many tags or none
- **Flat structure** — No nesting or hierarchy
- **Built-in favorite** — System tag for quick access

### Use Cases

| Tag | Purpose |
|-----|---------|
| `favorite` | Frequently accessed repositories |
| `active` | Currently working on |
| `needs-review` | Has changes needing attention |
| `on-hold` | Paused but not archived |
| `client-work` | Billable hours |
| `learning` | Tutorial or experiment projects |
| `archived` | Deprecated but kept for reference |

### Built-in: Favorite

The `favorite` tag is a system tag with special handling:

- **Quick filter** — Filter dashboard to favorites only
- **Sidebar shortcut** — Quick access section (if enabled)
- **Default filter** — Some views default to favorites when nothing selected

### Managing Tags

**CLI:**

```bash
# List all tags in use
gitty tag list

# Add a tag to a repository
gitty tag add <repo-uuid> favorite

# Remove a tag from a repository
gitty tag remove <repo-uuid> favorite

# Filter by tag
gitty filter --tag favorite
```

**GUI:**

- Add/remove tags inline on repository detail pages
- Use tag filter dropdown on dashboard

## Groups vs Tags: When to Use

| Aspect | Groups | Tags |
|--------|--------|------|
| Cardinality | Exactly one | Zero or more |
| Structure | Hierarchical tree | Flat list |
| Purpose | "Where it lives" | "What state it's in" |
| Analogy | Folders | Labels |
| Filtering | Exact match or subtree | Any match |
| Example | `work/project-a` | `favorite`, `active` |

### Decision Guide

**Use Groups when:**
- Organizing by project, team, or client
- Creating a navigable tree structure
- Separating work and personal repositories

**Use Tags when:**
- Marking work-in-progress status
- Flagging for review or attention
- Creating cross-cutting categories (like `favorite`)

**Combine both:**

```
work/project-a/api          [favorite, active]
work/project-a/frontend     [active, needs-review]
personal/dotfiles           [favorite]
```

## Filtering

The real power comes from combining Groups and Tags in filters.

**CLI:**

```bash
# Filter by group
gitty filter --group work/project-a

# Filter by tag
gitty filter --tag favorite

# Combined
gitty filter --group work --tag active
```

**GUI:**

- Use the dashboard filter bar
- Click a group in the sidebar tree
- Select tags from the dropdown

**Macro Targeting:**

Filters become selection criteria for macros:

```bash
# Run macro on filtered results
gitty macro run "Daily Fetch" --group work --tag active
```

## Best Practices

### Group Strategy

1. **Start flat** — Don't over-nest initially; 2-3 levels maximum is usually enough
2. **Use consistent naming** — `work/...` for professional, `personal/...` for side projects
3. **Mirror your mental model** — Groups should match how you think about your projects
4. **Review periodically** — Archive or delete groups that are no longer relevant

### Tag Strategy

1. **Keep it simple** — Too many tags become meaningless; 5-10 tags is usually plenty
2. **Use verbs for status** — `active`, `needs-review`, `on-hold`
3. **Use nouns for categories** — `client-work`, `learning`, `infrastructure`
4. **Clean up regularly** — Tags like `needs-review` should be temporary

### Migration

If you need to reorganize:

1. Plan the new structure on paper first
2. Create new groups
3. Move repositories in batches
4. Verify everything is where you expect
5. Delete old groups

## See Also

- [Repository](repository.md) — Repository management
- [Macros](macros.md) — Using filters in automation
- [CLI Reference](../cli/organization.md) — Organization commands