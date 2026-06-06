# CLI Reference

Auto-generated from `gitty --help`. Regenerate with:

```bash
./scripts/generate-cli-reference.sh
```

## gitty

```text
Workspace synchronization and orchestration for Git repositories

Usage: gitty.exe <COMMAND>

Commands:
  scan          Scan a directory for Git repositories and register them
  list          List all registered repositories
  status        Show the Git status of each registered repository
  fetch         Fetch all remotes for every registered repository (or a single one)
  pull          Pull every registered repository (or a single one)
  checkout      Checkout a branch in every registered repository (or a single one)
  group         Manage Groups
  tag           Manage Tags
  filter        Filter repositories by Group or Tag
  macro         Manage and run Macros
  health        Show workspace and repository health status
  scheduler     Manage the background scheduler
  notification  Manage notification settings
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## gitty scan

```text
Scan a directory for Git repositories and register them

Usage: gitty.exe scan <PATH>

Arguments:
  <PATH>  Directory to scan recursively for `.git` repositories

Options:
  -h, --help  Print help
```

## gitty list

```text
List all registered repositories

Usage: gitty.exe list

Options:
  -h, --help  Print help
```

## gitty status

```text
Show the Git status of each registered repository

Usage: gitty.exe status

Options:
  -h, --help  Print help
```

## gitty fetch

```text
Fetch all remotes for every registered repository (or a single one)

Usage: gitty.exe fetch [REPO]

Arguments:
  [REPO]  Optional repository path or directory name to target

Options:
  -h, --help  Print help
```

## gitty pull

```text
Pull every registered repository (or a single one)

Usage: gitty.exe pull [REPO]

Arguments:
  [REPO]  Optional repository path or directory name to target

Options:
  -h, --help  Print help
```

## gitty checkout

```text
Checkout a branch in every registered repository (or a single one)

Usage: gitty.exe checkout [OPTIONS] <BRANCH>

Arguments:
  <BRANCH>  The branch name to check out

Options:
      --repo <REPO>  Optional repository path or directory name to target
  -h, --help         Print help
```

## gitty group

```text
Manage Groups

Usage: gitty.exe group <COMMAND>

Commands:
  list    List all Groups
  create  Create a new Group
  rename  Rename an existing Group
  delete  Delete a Group (repos move to Ungrouped)
  assign  Assign a Repository to a Group
  tree    Show the Group tree hierarchy
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## gitty tag

```text
Manage Tags

Usage: gitty.exe tag <COMMAND>

Commands:
  list    List all Tags in use
  add     Add a Tag to a Repository
  remove  Remove a Tag from a Repository
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## gitty filter

```text
Filter repositories by Group or Tag

Usage: gitty.exe filter [OPTIONS]

Options:
      --group <GROUP>  Filter by Group name or id
      --tag <TAG>      Filter by Tag name
  -h, --help           Print help
```

## gitty macro

```text
Manage and run Macros

Usage: gitty.exe macro <COMMAND>

Commands:
  list    List all defined Macros
  define  Define a new Macro from inline steps
  delete  Delete a Macro
  show    Show steps of a Macro
  run     Run a Macro against a selection of repositories
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## gitty health

```text
Show workspace and repository health status

Usage: gitty.exe health [OPTIONS]

Options:
      --repo <REPO>  Show details for a single repository (path, name, or UUID)
  -h, --help         Print help
```

## gitty scheduler

```text
Manage the background scheduler

Usage: gitty.exe scheduler <COMMAND>

Commands:
  start   Start the background scheduler
  stop    Stop the background scheduler
  status  Show scheduler status
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## gitty notification

```text
Manage notification settings

Usage: gitty.exe notification <COMMAND>

Commands:
  show  Show current notification configuration and history
  set   Set the notification trigger mode
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### gitty notification show

```text
Show current notification configuration and history

Usage: gitty.exe notification show

Options:
  -h, --help  Print help
```

### gitty notification set

```text
Set the notification trigger mode

Usage: gitty.exe notification set <MODE>

Arguments:
  <MODE>  Trigger mode: on_critical, on_any_change, on_scheduler_complete, disabled

Options:
  -h, --help  Print help
```
