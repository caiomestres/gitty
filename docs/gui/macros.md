# Macros View

The **Macros** page provides visual management of automation sequences — create, edit, and execute macros with a graphical step editor.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Macros                                            [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│  [+ Create Macro]  [Import ▼]  [Filter: All ▼]             │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Macro List                                           │ │
│  │ ═══════════════════════════════════════════════════  │ │
│  │                                                      │ │
│  │ 📋 Morning Sync                              [Run ▼] │ │
│  │    2 steps  •  Last run: today  •  15 repos         │ │
│  │                                                      │ │
│  │ 📋 Deploy to Staging                           [Run] │ │
│  │    4 steps  •  Never run                            │ │
│  │                                                      │ │
│  │ 📋 Clean and Rebuild                           [Run] │ │
│  │    3 steps  •  Last run: 3 days ago                  │ │
│  │                                                      │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │ Macro Editor: Morning Sync                            │ │
│  │ ═══════════════════════════════════════════════════  │ │
│  │                                                      │ │
│  │ Name: [Morning Sync                      ]          │ │
│  │                                                      │ │
│  │ Steps:                                              │ │
│  │ ┌──────────────────────────────────────────────────┐│ │
│  │ │ 1.  Git Operation: fetch                        ││ │
│  │ │     [Edit] [▲] [▼] [×]                          ││ │
│  │ └──────────────────────────────────────────────────┘│ │
│  │ ┌──────────────────────────────────────────────────┐│ │
│  │ │ 2.  Git Operation: pull                         ││ │
│  │ │     Condition: if_behind                        ││ │
│  │ │     [Edit] [▲] [▼] [×]                          ││ │
│  │ └──────────────────────────────────────────────────┘│ │
│  │                                                      │ │
│  │ [+ Add Step]                                        │ │
│  │                                                      │ │
│  │ ─────────────────────────────────────────────────────  │ │
│  │                                                      │ │
│  │ Rollback Steps (optional):                          │ │
│  │ (None defined)                                      │ │
│  │                                                      │ │
│  │ [+ Add Rollback Step]                               │ │
│  │                                                      │ │
│  │ [Save] [Save & Run] [Delete]                        │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Macro List

### List Columns

| Column | Description |
|--------|-------------|
| **Name** | Macro name (click to edit) |
| **Steps** | Count of steps in sequence |
| **Last run** | Relative time or "Never" |
| **Repositories** | Target count from last run |
| **Run button** | Quick execute |

### List Sorting

Sort by:
- **Name** (alphabetical)
- **Last run** (most recent first)
- **Step count** (complexity)

### Built-in Macros

System macros shown with 🔒 icon:

```
🔒 __scheduler_default    1 step   System
   └─ Used by scheduler for background fetch
```

Cannot edit or delete, but can duplicate as custom macro.

## Creating Macros

### Create Dialog

```
Create Macro
════════════

Name: [Morning Sync              ]

Template: [Blank ▼]
        ├─ Blank
        ├─ Daily Sync (fetch, pull)
        ├─ Clean Build (clean, build)
        └─ From existing macro...

[Cancel]  [Create]
```

Templates provide common patterns as starting points.

## Step Editor

### Step Types

| Type | Icon | Description |
|------|------|-------------|
| **Git Operation** | 🌿 | Git commands (fetch, pull, checkout) |
| **Shell Command** | 💻 | Arbitrary shell execution |

### Adding Steps

Click **+ Add Step** to open:

```
Add Step
════════

Type: [Git Operation ▼]  [Shell Command]

Git Operation:
  Operation: [fetch ▼]
            ├─ fetch
            ├─ pull
            └─ checkout

  For checkout:
  Branch: [main                    ]

[Cancel]  [Add Step]
```

### Shell Command

```
Add Step
════════

Type: [Shell Command]

Command: [npm ci                        ]

Working directory:
  ( ) Repository root (default)
  ( ) Custom: [___________________]

[Cancel]  [Add Step]
```

### Step Configuration

Each step can be configured:

```
Edit Step: Git Operation
═══════════════════════════

Operation: [checkout ▼]
Branch:    [main           ]

Condition (optional):
  [☐ Only if condition is met]
      └─ [if_not_on_branch ▼]
          ├─ if_dirty
          ├─ if_clean
          ├─ if_ahead
          ├─ if_behind
          └─ if_not_on_branch: [main]

Retry configuration:
  [☐ Retry on transient failures]
      ├─ Max attempts: [3  ]
      └─ Backoff: [5   ] seconds

[Cancel]  [Save Step]
```

### Step Reordering

Use ▲ ▼ buttons to reorder steps, or drag-and-drop:

```
1. fetch                 [▲] [▼] [×]
2. checkout:main   ←──── [▲] [▼] [×]  (dragging)
3. pull                  [▲] [▼] [×]
```

### Step Deletion

Click × to remove a step with confirmation:

```
Delete step?
═══════════

Remove "checkout:main" from the macro?

[Cancel]  [Delete]
```

## Conditions

Make steps conditional:

| Condition | Runs when... |
|-----------|--------------|
| `if_dirty` | Repository has uncommitted changes |
| `if_clean` | Repository has no uncommitted changes |
| `if_ahead` | Repository ahead of remote |
| `if_behind` | Repository behind remote |
| `if_detached` | In detached HEAD state |
| `if_on_branch:{name}` | Currently on specified branch |

### Condition Examples

```
Step: pull
Condition: if_behind

→ Only pulls if behind remote (avoids unnecessary merges)
```

```
Step: shell:git stash
Condition: if_dirty

→ Stashes changes before switching branches
```

## Variables

Define variables for reuse:

```
Variables
═══════════

Define variables that can be used in steps as {variable_name}.

┌──────────────────────────────────────┐
│ Name       │ Default  │ Required    │
│ ───────────┼──────────┼─────────────│
│ branch     │ main     │ ☐           │
│ environment│ staging  │ ☑           │
└──────────────────────────────────────┘

[+ Add Variable]
```

### Variable Usage

Reference in steps:

```
Step: checkout:{branch}
Step: shell:./deploy.sh {environment}
```

Runtime values:
```bash
gitty macro run "Deploy" --var branch=feature/login --var environment=production
```

## Rollback Steps

Define recovery actions if main steps fail:

```
Rollback Steps
═══════════════

If main steps fail, execute:

1. shell:git reset --hard HEAD@{1}
   (Restore to state before failed operation)

2. shell:git stash pop
   (Restore stashed changes)

[+ Add Rollback Step]
```

**Note:** Not all operations can be rolled back. Test macros before relying on rollback.

## Saving Macros

### Save

Save changes without running:

```
[Save]  [Save & Run]  [Discard Changes]
```

### Save & Run

Save and immediately execute:

```
Save & Run
═══════════

Macro "Morning Sync" saved.

Run now?
[Target selection dialog appears]

[Cancel]  [Run]
```

## Running Macros

### Target Selection

```
Run Macro: Morning Sync
════════════════════════

Select repositories to target:

(•) All repositories (15)
( ) Specific repositories
    [☐] myapp
    [☐] api
    [☐] web
    ...
( ) Group
    [work ▼]
( ) Tag
    [favorite ▼]

Options:
  [☐ Confirm before executing]
  [☐ Stop on first failure]

[Cancel]  [Run]
```

### Execution Panel

Once running:

```
Running: Morning Sync
═══════════════════════

Target: 15 repositories
Progress: 8/15 complete

┌──────────────────────────────────────┐
│ Repository   │ Step 1  │ Step 2    │
│ ─────────────┼─────────┼───────────│
│ myapp        │ ✓       │ ✓         │
│ api          │ ✓       │ running   │
│ web          │ ✓       │ pending   │
│ docs         │ ✓       │ pending   │
│ utils        │ failed  │ skipped   │
│ ...          │ ...     │ ...       │
└──────────────────────────────────────┘

[Cancel Remaining]  [View Logs]
```

### Results

On completion:

```
Completed: Morning Sync
════════════════════════

Duration: 45 seconds
Results:
  Success: 14 repositories
  Failed:  1 repository

Failed:
• utils — Step 2: merge conflict
  └─ Error: Automatic merge failed; fix conflicts

[View Details]  [Run Again]  [Close]
```

## Import/Export

### Import

```
Import Macro
════════════

Paste macro JSON:

[                                              ]
[                                              ]
[                                              ]

[Load from file...]

[Cancel]  [Import]
```

### Export

```
Export Macro: Morning Sync
════════════════════════════

Save as JSON for sharing or backup.

[Copy to Clipboard]  [Save to File...]

⚠️ Warning: May contain paths or sensitive data.
   Review before sharing.
```

## Deleting Macros

```
Delete Macro: Deploy to Staging
════════════════════════════════

⚠️ This will permanently delete the macro
   "Deploy to Staging" (4 steps).

This cannot be undone.

[Cancel]  [Delete Macro]
```

**Note:** Built-in system macros cannot be deleted.

## Best Practices

### Naming

| Good | Less Good |
|------|-----------|
| "Morning Sync" | "Macro 1" |
| "Deploy to Staging" | "Stuff" |
| "Clean and Rebuild" | "clean" |
| "Feature Branch Update" | "fb-update" |

### Step Safety

1. **Test on one repo first** — Use `--repo` to test
2. **Use conditions** — Don't blindly pull dirty repos
3. **Order matters** — Fetch before checkout, checkout before pull
4. **Add rollbacks** — For destructive operations

### Variable Design

```
Good variables:
• branch — allows targeting any branch
• environment — staging vs production

Avoid:
• repo-specific paths
• Personal directory names
• Hardcoded credentials
```

## CLI Equivalent

```bash
# List macros
gitty macro list

# Define
gitty macro define "Morning Sync" fetch pull

# With condition
gitty macro define "Smart Pull" "pull (if_behind and if_clean)"

# Show
gitty macro show "Morning Sync"

# Run
gitty macro run "Morning Sync"

# Run on subset
gitty macro run "Morning Sync" --group work

# Delete
gitty macro delete "Morning Sync"
```

## Troubleshooting

### Macro fails on specific repo

1. Test on that repo alone: `--repo <name>`
2. Check repo status: `gitty status --repo <name>`
3. Review job output for specific error

### Condition not working

1. Verify repository actually matches condition
2. Check syntax: `(if_dirty)` not `if_dirty`
3. Test without condition first

### Variables not substituted

1. Check variable name matches exactly (case-sensitive)
2. Verify braces used: `{var}` not `var`
3. Ensure variable provided at runtime

### Slow execution

1. Check for many repositories (sequential execution)
2. Look for slow shell commands
3. Consider if network issues affect Git operations

## See Also

- [Macros Concepts](../concepts/macros.md) — Automation theory
- [Scheduler](../concepts/scheduler.md) — Automated macro execution
- [CLI Reference](../cli/automation.md) — Command-line macros