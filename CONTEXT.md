# Gitty

Synchronization, orchestration, and workspace health platform for developers managing large collections of Git repositories.

## Language

### Core Entities

**Workspace**:
A named collection of one or more Scan Roots whose repositories are managed as a single unit. Has one health score, one dashboard, one set of Groups and Tags. For v1, a single implicit default Workspace exists.
_Avoid_: project, environment, profile

**Scan Root**:
A filesystem directory that Gitty scans recursively for `.git` directories. A Workspace contains one or more Scan Roots. Repositories discovered under different Scan Roots are merged into the same Workspace.
_Avoid_: root folder, base path, search directory

**Repository**:
A local Git repository discovered by scanning a Scan Root for `.git` directories. The fundamental unit that Gitty operates on. Identified by a Gitty-assigned UUID stored in Config; the filesystem path is recorded but identity survives moves via re-linking.
_Avoid_: repo, project, service

**Re-linking**:
The process that preserves a Repository's identity (UUID, Group, Tags) when its filesystem path changes. On rescan, a Repository whose recorded path no longer exists is marked Missing; if exactly one newly-discovered repository shares its content fingerprint, the recorded path is updated and the identity preserved. Ambiguous matches — the same fingerprint shared by clones or forks — are never auto-linked.
_Avoid_: re-attach, rebind, reconnect

**Missing**:
A Repository state indicating its recorded filesystem path no longer exists on disk. Missing Repositories are retained in the registry — never silently deleted — so identity and organization survive relocation or temporary disappearance.
_Avoid_: lost, orphaned, stale

### Organization

**Group**:
A hierarchical organizational category for Repositories. Supports arbitrary nesting (tree structure). A Repository belongs to exactly one Group. Newly discovered Repositories are assigned to a default "Ungrouped" Group.
_Avoid_: folder, category, collection

**Tag**:
An additive label attached to a Repository. A Repository can have zero or more Tags. "Favorite" is a built-in system Tag.
_Avoid_: label, flag, marker

### Operations

**Macro**:
A named, ordered sequence of Steps that can target any selection of Repositories (one repo, a Group, a Tag filter, or all). All operations in Gitty are Macros — single-command operations like `gitty pull` are single-step Macros. Supports variables, conditions, rollback, and confirmations.
_Avoid_: script, workflow, pipeline, bulk operation

**Step**:
A single unit of work inside a Macro. Either a typed Git Operation (with structured parameters, progress tracking, and error classification) or a Shell Command (arbitrary command string, less structure).
_Avoid_: command, action, task

**Git Operation**:
A first-class Step type representing a Git command (pull, fetch, checkout, rebase, etc.) with structured parameters. Executed via shell-out to `git` CLI for full compatibility. Supports smart retry on transient/network errors.
_Avoid_: git command, git action

**Shell Command**:
A Step type representing an arbitrary shell command string (e.g., `mvn clean install`, `docker compose restart`). Never auto-retried (Gitty cannot determine idempotency).
_Avoid_: custom command, raw command

**Job**:
One Macro execution on one Repository. The unit of scheduling, retry, and status reporting. States: Pending, Running, Success, Failed, Skipped, Cancelled. Steps are tracked within the Job.
_Avoid_: task, work item, run

### Health

**Health Check**:
An evaluation of a single Repository against a specific criterion (freshness, divergence, conflicts, dirty tree, detached HEAD, etc.). Produces a status: healthy, warning, or critical.
_Avoid_: diagnostic, assessment

**Workspace Health**:
An aggregate score derived from per-Repository Health Checks. Calculated as percentage of Repositories not in critical state. Subsumes the concepts of "AI Readiness" and "Drift Detection" — those are categories of Health Checks, not separate features.
_Avoid_: workspace score, readiness score, drift score

### Infrastructure

**Config**:
User-level configuration file resolved via `dirs::config_dir()` (`%APPDATA%\gitty\` on Windows, `~/Library/Application Support/gitty/` on macOS, `~/.config/gitty/` on Linux). Contains Workspace definitions, Scheduler rules, Macro definitions, and user preferences.
_Avoid_: settings, preferences file

**Scheduler**:
A background automation engine that runs Macros when conditions are met. Default action is `git fetch --all`; configurable to run any Macro. Trigger conditions include time-of-day, day-of-week, and power source.
_Avoid_: cron, timer, automation engine

**Lock**:
A file-level lock preventing the CLI and GUI from running conflicting operations on the same Repository simultaneously. Stored centrally in the Config directory.
_Avoid_: mutex, semaphore

**Notification**:
A timestamped record surfaced to the user when a notable event occurs (Health Check entering critical, Scheduler run completing, etc.). Delivered via OS-native toast for critical severity and displayed in an in-app panel for all severities. Stored as a bounded list in Config with a 7-day TTL. Triggers are user-configurable.
_Avoid_: alert, event, message

### Deferred (v2)

**Dependency Map**:
A graph of inter-Repository relationships discovered by parsing manifest and infrastructure files (pom.xml, Gradle, package.json, Docker Compose, Kubernetes, Helm). Enables cascading health warnings, smart macro targeting, and architecture visualization. Deferred from v1 — core features work without it.
_Avoid_: architecture graph, service map
