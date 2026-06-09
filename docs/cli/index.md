# CLI Reference Overview

Gitty's command-line interface (`gitty-cli`) provides complete workspace management from the terminal. All GUI features are accessible via CLI, making it ideal for scripting, automation, and server environments.

## Usage Pattern

```bash
gitty <command> [subcommand] [options] [arguments]
```

## Command Categories

| Category | Commands | Purpose |
|----------|----------|---------|
| **Core** | `scan`, `list`, `status`, `fetch`, `pull`, `checkout` | Repository operations |
| **Organization** | `group`, `tag`, `filter` | Groups, tags, filtering |
| **Automation** | `macro`, `scheduler` | Macros and background tasks |
| **Health** | `health`, `notification` | Monitoring and alerts |
| **System** | `config`, `activity` | Settings and logging |

## Global Options

```bash
gitty --help              # Show help
gitty --version           # Show version
gitty -v, --verbose       # Verbose output
gitty -q, --quiet         # Suppress non-error output
```

## Help

Get help for any command:

```bash
gitty --help              # Top-level help
gitty scan --help         # Command-specific help
gitty group --help        # Subcommand help
gitty group create --help # Specific subcommand help
```

## Configuration Access

CLI shares configuration with GUI:

| Platform | Config Location |
|----------|-----------------|
| Windows | `%APPDATA%\gitty\config.json` |
| macOS | `~/Library/Application Support/gitty/config.json` |
| Linux | `~/.config/gitty/config.json` |

Changes made via CLI immediately affect GUI (and vice versa).

## Output Formats

Most commands support output format selection:

```bash
# Human-readable (default)
gitty list

# JSON for scripting
gitty list --format json

# TSV for spreadsheets
gitty list --format tsv
```

Formats:
- `human` — Human-readable tables
- `json` — Machine-parseable JSON
- `tsv` — Tab-separated values

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments |
| `3` | Repository error |
| `4` | Configuration error |
| `5` | Network error |
| `130` | Interrupted (Ctrl-C) |

## Interactive vs Scripting

**Interactive use:**
- Default human-readable output
- Progress bars where applicable
- Confirmation prompts for destructive operations
- Colored output

**Scripting use:**
- `--format json` for parsing
- `--quiet` to suppress progress
- `--yes` to skip confirmations
- Exit code checking

## Environment Variables

| Variable | Effect |
|------------|--------|
| `GITTY_CONFIG_DIR` | Override config directory |
| `GITTY_LOG_LEVEL` | Set logging level (debug, info, warn, error) |
| `GITTY_NO_COLOR` | Disable colored output |
| `GIT_TERMINAL_PROMPT` | Passed to Git (set to `0` to disable prompts) |

## CLI-Only Features

Some features are CLI-only:

- **Daemon mode** — Scheduler as background process
- **Batch operations** — Easier for large-scale scripting
- **Cron integration** — Direct scheduler for automation
- **Pipe integration** — Chain with other Unix tools

## GUI-Only Features

Some features require GUI:

- **Visual theme switching** — CLI can set, but preview in GUI
- **In-app notifications** — OS toasts available in GUI
- **Interactive macro builder** — Visual step editor
- **Repository drag-and-drop** — Reorganize via GUI

## Quick Reference

### Daily Commands

```bash
gitty scan ~/projects        # Discover repositories
gitty list                   # Show all repos
gitty status                 # Check status
gitty fetch                  # Fetch all remotes
gitty pull                   # Pull all repos
gitty health                 # Check workspace health
```

### Organization

```bash
gitty group list             # List groups
gitty group create work      # Create group
gitty group assign <id> work # Assign repo to group
gitty tag add <id> favorite  # Tag repository
gitty filter --group work    # Filter by group
```

### Automation

```bash
gitty macro define sync fetch pull   # Create macro
gitty macro run sync                 # Run macro
gitty scheduler start                # Start daemon
```

## Next Steps

- [Core Commands](core.md) — Daily operations
- [Organization Commands](organization.md) — Groups and tags
- [Automation Commands](automation.md) — Macros and scheduler
- [Full CLI Reference](../cli-reference.md) — Auto-generated reference