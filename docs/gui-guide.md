# GUI Guide

Gitty's desktop app provides the same capabilities as the CLI through a visual interface. This guide covers each main page.

## Dashboard

The landing page after launch. Shows workspace summary metrics, repository cards, and quick actions.

- **Health score** — aggregate workspace health at a glance
- **Repository grid** — each card shows name, branch, dirty indicator, and health color
- **Fetch All** — triggers a bulk fetch across every registered repository
- **Search and filter** — narrow the grid by name, group, or tag

Click a repository card to open its detail view with branch info, remotes, and recent commits.

## Health

Dedicated health monitoring view with traffic-light status per repository.

- **Workspace score** — percentage of repos not in critical state
- **Per-repo breakdown** — individual health checks (freshness, divergence, dirty tree, detached HEAD)
- **Sort and filter** — surface critical repos first
- **Drill-down** — click a repo for detailed check results and recommended actions

Health data is cached in `health.json` alongside your config and refreshed on scan, fetch, and scheduler runs.

## Changes

The change dashboard answers "what changed across my workspace?"

- **Time windows** — 24 hours, 7 days, or 30 days
- **Grouping** — by author, repository, or branch
- **Commit list** — message, author, date, and affected repo for each entry

Useful for standup prep, code review triage, and spotting stale repos.

## Groups

Manage the hierarchical group tree.

- **Tree view** — nested groups with drag-and-drop reassignment (where supported)
- **Create / rename / delete** — organize repos into categories
- **Assign repos** — move a repository to a group from the tree or repo detail view
- **Ungrouped** — default bucket for newly discovered repositories

## Macros

Define and run named command sequences.

- **Macro list** — all defined macros with step counts
- **Define macro** — compose steps: `fetch`, `pull`, `checkout:branch`, `shell:command`
- **Run macro** — target all repos, a group, a tag filter, or a single repo
- **Execution log** — per-repo job status (pending, running, success, failed)

Built-in macros include fetch-all and pull-all shortcuts accessible from the Dashboard.

## Settings

Application configuration and scan management.

- **Scan roots** — add, remove, and trigger rescans
- **Scheduler** — enable/disable, configure time windows and power-source rules
- **Notifications** — set trigger mode (critical, any change, scheduler complete, disabled)
- **Preferences** — theme, startup behavior, and paths

Config is stored at the platform-native location (`%APPDATA%\gitty\` on Windows, `~/Library/Application Support/gitty/` on macOS, `~/.config/gitty/` on Linux).
