# Concepts

Gitty uses precise domain language. This page defines the core terms you will encounter in the GUI, CLI, and configuration.

## Workspace

A named collection of one or more **Scan Roots** whose repositories are managed as a single unit. A workspace has one health score, one dashboard, and one set of groups and tags. In v1, a single implicit default workspace exists.

## Repository

A local Git repository discovered by scanning a Scan Root for `.git` directories. The fundamental unit Gitty operates on. Identified by a Gitty-assigned UUID stored in config; the filesystem path is recorded but identity survives moves via **re-linking**.

**Re-linking** preserves a repository's UUID, group, and tags when its path changes. On rescan, a missing path triggers fingerprint matching against newly discovered repos.

## Group

A hierarchical organizational category for repositories. Supports arbitrary nesting (tree structure). A repository belongs to exactly one group. Newly discovered repositories are assigned to a default **Ungrouped** group.

Use groups to mirror your mental model — e.g. `work/backend`, `personal/dotfiles`.

## Tag

An additive label attached to a repository. A repository can have zero or more tags. **Favorite** is a built-in system tag for quick filtering.

Unlike groups, tags are cross-cutting — one repo can carry `favorite`, `active`, and `needs-review` simultaneously.

## Macro

A named, ordered sequence of **Steps** that targets any selection of repositories (one repo, a group, a tag filter, or all). All operations in Gitty are macros — even `gitty pull` is a single-step macro.

Macros support variables, conditions, rollback, and confirmations for safe bulk operations.

## Health Check

An evaluation of a single repository against a specific criterion:

| Criterion | What it detects |
|-----------|-----------------|
| Freshness | Last fetch/pull age |
| Divergence | Ahead/behind remote |
| Dirty tree | Uncommitted changes |
| Detached HEAD | Not on a branch |

Each check produces **healthy**, **warning**, or **critical** status. **Workspace Health** is the aggregate percentage of repositories not in critical state.

## Scheduler

A background automation engine that runs macros when conditions are met. The default action is `git fetch --all`; you can configure it to run any macro.

Trigger conditions include:

- Time of day and day of week
- Power source (pauses on battery when configured)

The scheduler runs as a daemon process with a PID file alongside your config.

## Notification

A timestamped alert for health changes, scheduler completion, or workspace events. Configurable triggers:

- **on-critical** — only critical health changes
- **on-any-change** — any health status change
- **on-scheduler-complete** — after a scheduled run finishes
- **disabled** — no notifications

Delivered via the in-app notification panel and OS-native toast support where available.
