# Settings

The **Settings** page provides configuration for scan roots, scheduler, notifications, themes, and application preferences.

## Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Settings                                          [⚙️]    │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┬──────────────────────────────────────────┐    │
│  │          │                                          │    │
│  │ Scan     │  Scan Roots                              │    │
│  │ Roots    │  ══════════════════════════════════════  │    │
│  │          │                                          │    │
│  │ Scheduler│  📁 /home/user/projects                   │    │
│  │          │    42 repositories    [Rescan] [Remove] │    │
│  │ Notifi-  │                                          │    │
│  │ cations  │  📁 /home/user/experiments                │    │
│  │          │    5 repositories     [Rescan] [Remove] │    │
│  │ Theme    │                                          │    │
│  │          │  [+ Add Scan Root]                        │    │
│  │ Privacy  │                                          │    │
│  │          │  ───────────────────────────────────────  │    │
│  │ Advanced │                                          │    │
│  │          │  Scheduler                                │    │
│  │          │  ══════════════════════════════════════  │    │
│  │          │                                          │    │
│  │          │  Enable scheduler: [☑]                  │    │
│  │          │                                          │    │
│  │          │  Trigger: [Simple ▼]                      │    │
│  │          │  Interval: [60  ] minutes                 │    │
│  │          │                                          │    │
│  │          │  [Save Changes]                           │    │
│  │          │                                          │    │
│  └──────────┴──────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Settings Sections

### Scan Roots

Manage directories Gitty scans for repositories.

#### Current Scan Roots

```
Scan Roots
═══════════

📁 /home/user/projects
   42 repositories discovered
   Last scan: 2 hours ago
   [Rescan] [Remove]

📁 /home/user/experiments
   5 repositories discovered
   Last scan: 3 days ago
   [Rescan] [Remove]
```

**Per-root actions:**
- **Rescan** — Re-scan for new/moved repositories
- **Remove** — Remove from Gitty (does not delete files)

#### Adding Scan Roots

**Add button:**

```
Add Scan Root
═══════════════

Choose method:

[Browse...]  Select folder via native file picker

-or-

Path: [_________________________]
      [Add]
```

**Drag and drop:**

Drop a folder onto the Settings page to add as scan root.

**Validation:**

| Error | Meaning |
|-------|---------|
| "Not a directory" | Path is a file, not folder |
| "Permission denied" | Cannot read directory |
| "Already exists" | Already a scan root |
| "Nested scan root" | Inside existing scan root |

#### Scanning

When you click **Rescan**:

```
Scanning: /home/user/projects
═══════════════════════════════

Walking directory tree...
Found 42 Git repositories
Identifying existing repositories...
Registering new repositories: 3
Re-linking moved repositories: 1

Complete!

[Close]
```

### Scheduler

Configure background automation.

#### Enable/Disable

```
Scheduler
═══════════

Enable scheduler: [☑]

The scheduler runs macros automatically
in the background when conditions are met.
```

**GUI:** Scheduler runs as tokio task (only while app open)
**CLI:** Scheduler runs as daemon (persistent)

#### Trigger Configuration

**Simple trigger:**

```
Trigger: [Simple ▼]

Run every: [60  ] minutes
```

**Advanced trigger:**

```
Trigger: [Advanced ▼]

Run every: [30  ] minutes

Only during time window:
  From: [09:00]
  To:   [18:00]

Only on days:
  [☑] Mon  [☑] Tue  [☑] Wed  [☑] Thu  [☑] Fri  [☐] Sat  [☐] Sun
```

#### Power Policy

```
Power policy: [Run Always ▼]
            ├─ Run Always
            ├─ AC Power Only
            └─ Battery Threshold: [25  ]%
```

| Policy | Behavior |
|--------|----------|
| **Run Always** | Execute regardless of power state |
| **AC Power Only** | Skip when running on battery |
| **Battery Threshold** | Skip when battery below % |

#### Macro Selection

```
Macro to run: [__scheduler_default ▼]
            ├─ __scheduler_default (built-in fetch)
            ├─ Morning Sync
            └─ Custom Macro...
```

Select which macro executes on schedule. Default is `fetch` for all repositories.

### Notifications

Configure alert behavior.

#### Triggers

```
Notifications
═══════════════

Trigger when:
(•) Any health status changes
( ) Only critical health changes
( ) Only scheduler completions
( ) Disabled
```

| Trigger | Fires when... |
|---------|---------------|
| **Any change** | Healthy→Warning, Warning→Critical, etc. |
| **Critical only** | Only when status becomes Critical |
| **Scheduler complete** | After scheduled macro finishes |
| **Disabled** | Never |

#### Delivery

```
Delivery methods:

[☑] In-app notification panel
[☑] OS-native toast notifications

(Toast notifications require system permission)
```

**OS Support:**
- **macOS** — Native notification center
- **Windows** — Action Center
- **Linux** — Freedesktop notifications (GNOME, KDE, etc.)

### Theme

Select and preview visual themes.

```
Theme
════════

Select a theme:

┌──────────────────────────────────────────────────────┐
│ ┌────────────┐  ┌────────────┐  ┌────────────┐      │
│ │  🦁        │  │  🦁        │  │  🦁        │      │
│ │  Default   │  │   Dark     │  │   Brasil   │      │
│ │  ✓         │  │            │  │            │      │
│ └────────────┘  └────────────┘  └────────────┘      │
│     Warm           Dark          Green            │
│     Cream          Mode          Yellow           │
│                                  Blue             │
└──────────────────────────────────────────────────────┘

Active: Default
Change applies immediately.
```

**Preview cards** show:
- Mascot with theme colors
- Theme name
- Color palette description
- Checkmark for active theme

Click any card to switch immediately. Preference persists across restarts.

### Privacy

View privacy-related settings.

```
Privacy
═════════

Data Storage
───────────────

All data is stored locally on your machine:
• Config: /home/user/.config/gitty/
• Logs: /home/user/.config/gitty/logs/
• Cache: /home/user/.config/gitty/cache/

[Open Data Directory]

Network
─────────

Network connections made by Gitty:
[View Privacy Details] → /privacy.md
```

See [Privacy Policy](../privacy.md) for full details.

### Advanced

Additional configuration options.

```
Advanced
══════════

Activity Log
───────────────

Maximum entries: [1000 ▼]
              ├─ 100
              ├─ 500
              ├─ 1000 (default)
              ├─ 5000
              └─ 10000

Older entries are automatically removed.

Config
───────

Config file location:
/home/user/.config/gitty/config.json

[Export Config] [Reset to Defaults]

Danger Zone
────────────

⚠️ These actions cannot be undone.

[Clear All Data]  [Unregister All Repos]
```

## Saving Changes

Most settings save automatically. Some require explicit save:

```
[Save Changes]  [Discard]  [Reset to Default]
```

Unsaved changes indicated by dot on section name:

```
● Scheduler
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl/Cmd + ,` | Open Settings (from anywhere) |
| `Esc` | Close Settings |
| `1-6` | Jump to section |

## Troubleshooting

### Scan root not adding

1. Check path is valid directory
2. Verify read permissions
3. Ensure not already a scan root
4. Check not nested in existing scan root

### Scheduler not starting

1. Verify enabled toggle is on
2. Check trigger configuration
3. Try manual start: `gitty scheduler start`
4. Check logs for errors

### Theme not changing

1. Click theme card (not just hover)
2. Check if custom CSS interfering
3. Try restarting app
4. Verify config.json updated

### Notifications not showing

1. Check trigger level (not disabled)
2. Verify OS notification permissions
3. Test with manual health change
4. Check if Do Not Disturb active

## CLI Equivalent

```bash
# Scheduler
gitty scheduler start
gitty scheduler stop
gitty scheduler status
gitty scheduler set --enabled true --interval 60

# Notifications
gitty notification set on_critical
gitty notification show
```

Most settings configured via direct config editing (not recommended).

## See Also

- [Scheduler](../concepts/scheduler.md) — Background automation
- [Notifications](../concepts/domain.md#notification) — Alert system
- [Themes](themes.md) — Visual customization
- [Privacy](../privacy.md) — Full privacy details