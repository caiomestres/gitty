# Groups View

The **Groups** page and sidebar tree provide repository organization through hierarchical groups.

## Sidebar Group Tree

The sidebar displays a VS Code-style explorer:

```
┌──────────────┐
│ DASHBOARD    │
│ HEALTH       │
│ CHANGES      │
│ ACTIVITY     │
├──────────────┤
│ GROUPS       │
│              │
│ 📁 work       │
│   📁 backend │
│     🦁 api   │ ← Repository
│     🦁 workers│ ← Repository
│   📁 frontend│
│     🦁 web   │
│ 📁 personal   │
│   🦁 dotfiles │
│   🦁 blog    │
│ 📁 Ungrouped  │
│   🦁 temp-project│
└──────────────┘
```

### Tree Behavior

| Action | Behavior |
|--------|----------|
| **Click group** | Expand/collapse |
| **Click repository** | Open repository detail |
| **Right-click group** | Context menu (rename, delete, move) |
| **Right-click repo** | Context menu (favorite, fetch, move) |
| **Drag repo** | Move to another group (if enabled) |

### Visual Elements

| Element | Meaning |
|---------|---------|
| **📁** | Collapsed group |
| **📂** | Expanded group |
| **🦁** | Repository (with mascot icon) |
| **★** | Favorite repository indicator |
| **●** | Health status dot (color-coded) |

## Groups Admin Page

The dedicated Groups page provides full CRUD operations:

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Groups                                            [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│  [+ Create Group]  [View as Tree ▼]                        │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Tree View                                            │ │
│  │ ═══════════════════════════════════════════════════  │ │
│  │                                                      │ │
│  │ 📂 work (3 repos)                                    │ │
│  │   📂 backend (2 repos)                               │ │
│  │   📂 frontend (1 repo)                               │ │
│  │ 📂 personal (2 repos)                               │ │
│  │ 📂 Ungrouped (1 repo)                                │ │
│  │                                                      │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Group Details: work                                   │ │
│  │ ═══════════════════════════════════════════════════  │ │
│  │                                                      │ │
│  │ Name: [work                              ]            │ │
│  │ Parent: [None (root level)              ▼]           │ │
│  │                                                      │ │
│  │ Repositories: 3                                     │ │
│  │ • api                                               │ │
│  │ • workers                                           │ │
│  │ • web (in subgroup)                                 │ │
│  │                                                      │ │
│  │ [Rename] [Move] [Delete]                             │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Creating Groups

### Create Dialog

```
Create Group
════════════

Name: [work                        ]
      Tip: Use slashes for nesting (e.g., work/backend)

Parent: [None (root level)        ▼]
      ├─ None (root level)
      ├─ work
      │   ├─ backend
      │   └─ frontend
      └─ personal

[Cancel]  [Create]
```

**Nesting:**
- Use `/` in name to auto-create hierarchy: `work/backend`
- Or select parent from dropdown
- Both create identical structure

### Validation

| Rule | Error |
|------|-------|
| Name required | "Group name is required" |
| Duplicate at level | "A group named 'work' already exists here" |
| Empty name | "Name cannot be empty" |
| Self-parent | "Cannot move group into itself" |

## Renaming Groups

```
Rename Group: work/backend
═══════════════════════════

Current name: work/backend
New name:     [work/services   ]

This will update paths for 2 repositories.

[Cancel]  [Rename]
```

**Behavior:**
- Updates group name in-place
- Repositories remain assigned
- Full path updates automatically

## Moving Groups

Reparent groups in the hierarchy:

```
Move Group: work/backend
═════════════════════════

Current location: work/backend
New parent:       [work           ▼]
                ├─ (root level)
                ├─ work
                │   ├─ backend ← Can't select self
                │   └─ frontend
                └─ personal

New path will be: work/backend (unchanged)

[Cancel]  [Move]
```

**Constraints:**
- Cannot move into own descendant (cycle prevention)
- Repositories move with group
- Subgroups move with parent

## Deleting Groups

```
Delete Group: work/backend
═══════════════════════════

⚠️ This will delete the group 'work/backend' and
   move 2 repositories to 'Ungrouped'.

Repositories affected:
• api
• workers

Subgroups will also be deleted:
• work/backend/legacy (empty)

[Cancel]  [Delete Group]
```

**Behavior:**
- Group removed from tree
- Repositories moved to **Ungrouped**
- Subgroups recursively deleted
- UUIDs and other metadata preserved

## Assigning Repositories

### From Repository Detail

On any repository page:

```
Group: [work/backend ▼]
     ├─ Ungrouped
     ├─ work
     │   ├─ backend ← current
     │   └─ frontend
     └─ personal

Moving from 'Ungrouped' to 'work/backend'
```

**Confirmation:**
- Shows old and new group
- Lists any subgroup change
- Updates immediately on selection

### From Sidebar

1. Right-click repository in tree
2. Select **Move to Group**
3. Choose target group
4. Confirm move

### Bulk Assignment (Future)

Select multiple repositories and assign to group:

```
[ ] myapp    [ ] api    [ ] web    [Move to Group ▼]
     ☑         ☑                       ├─ work/backend
                                      ├─ work/frontend
                                      └─ personal
```

## Group Tree View

### Display Options

```
View: [Tree ▼]
    ├─ Tree (hierarchical)
    ├─ Flat (alphabetical)
    └─ By repository count
```

### Tree Display

**Hierarchical (default):**
```
📂 work
  📂 backend
    🦁 api
    🦁 workers
  📂 frontend
    🦁 web
📂 personal
  🦁 dotfiles
  🦁 blog
```

**Flat:**
```
work
work/backend
work/frontend
personal
Ungrouped
```

**By count:**
```
Ungrouped (12 repos)
work (8 repos)
work/backend (5 repos)
personal (3 repos)
```

## Ungrouped

Special system group for unassigned repositories:

- Always exists
- Cannot be deleted
- Cannot be renamed
- New repositories default here

**Best practice:** Move repositories out of Ungrouped as part of onboarding workflow.

## Constraints

### Hierarchy Limits

- **Max depth:** 10 levels (practical limit, not enforced)
- **Max children:** No limit
- **Max total groups:** No limit

### Naming

- **Case-sensitive:** `Work` and `work` are different
- **Slashes:** Used for nesting, escaped if part of name
- **Length:** 1-100 characters
- **Characters:** Alphanumeric, hyphens, underscores, slashes

### Cycle Prevention

Gitty prevents circular group references:

```
work
└─ backend
   └─ work  ← Cannot create (would be cycle)
```

## Best Practices

### Group Strategy

| Pattern | Example | Best For |
|---------|---------|----------|
| By project | `work/project-a`, `work/project-b` | Multi-repo projects |
| By team | `backend/services`, `frontend/apps` | Team-based orgs |
| By client | `client-acme`, `client-globex` | Agencies |
| By type | `dotfiles`, `experiments`, `forks` | Personal |
| Hybrid | `work/active`, `personal/dotfiles` | Most common |

### Nesting Depth

- **Prefer flat:** 2-3 levels maximum
- **Avoid deep:** More than 5 levels is hard to navigate
- **Group by purpose:** Not by technical structure

### Naming Conventions

```
Good:          Avoid:
────────       ──────────
work/backend   Work_Backend
mobile/ios     Mobile.iOS
dotfiles       My Dot Files
```

- Lowercase with hyphens
- Short but descriptive
- Consistent terminology

## CLI Equivalent

```bash
# List groups
gitty group list

# Tree view
gitty group tree

# Create
gitty group create work/backend

# Rename
gitty group rename work/backend work/services

# Move (reparent)
gitty group move work/services work

# Delete
gitty group delete work/services

# Assign repository
gitty group assign <repo-uuid> work/services
```

## Troubleshooting

### Group not appearing

1. Check if collapsed in sidebar (click to expand)
2. Verify group wasn't deleted
3. Refresh the view

### Can't move repository

1. Check permissions on config file
2. Verify repository is not "Missing"
3. Try from CLI for error details

### Cycle error

The group you're trying to move is either:
- The target's ancestor (can't move into descendant)
- The target itself

Choose a different parent.

### Rename fails

- Check for duplicate name at target level
- Verify no illegal characters
- Ensure name not too long

## See Also

- [Groups & Tags Concepts](../concepts/organization.md) — Organization theory
- [Repository Detail](repository.md) — Per-repo group assignment
- [Dashboard](dashboard.md) — Group filtering