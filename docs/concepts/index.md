# Core Concepts

Gitty uses precise domain language. Understanding these concepts helps you navigate both the GUI and CLI effectively.

## Overview

Gitty manages a **Workspace** of **Repositories** organized into **Groups** and **Tags**. You automate operations with **Macros**, monitor health with **Health Checks**, and schedule background tasks with the **Scheduler**.

```mermaid
graph TD
    A[Workspace] --> B[Scan Roots]
    A --> C[Repositories]
    A --> D[Groups]
    A --> E[Tags]
    A --> F[Macros]
    A --> G[Scheduler]

    C --> H[Health Checks]
    C --> I[Liveness]
    C --> J[Activity Log]

    D --> C
    E --> C
    F --> C
```

## Key Concepts

### [Workspace](domain.md#workspace)

A named collection of one or more **Scan Roots** whose repositories are managed as a single unit. The workspace has one health score, one dashboard, and one set of groups and tags.

### [Repository](repository.md)

A local Git repository discovered by scanning for `.git` directories. Each repository is identified by a Gitty-assigned UUID that survives filesystem moves via **re-linking**.

### [Groups & Tags](organization.md)

- **Groups** — Hierarchical organizational categories (e.g., `work/backend`)
- **Tags** — Cross-cutting labels (e.g., `favorite`, `needs-review`)

### [Health](health.md)

Health checks evaluate repositories against criteria like freshness, divergence, dirty state, and detached HEAD. The **Workspace Health** score aggregates results.

### [Liveness](liveness.md)

HTTP endpoint monitoring for repositories with associated services. Tracks service availability independently from Git health.

### [Macros](macros.md)

Named sequences of **Steps** (Git operations and shell commands) that target repository selections. Support variables, conditions, rollback, and confirmations.

### [Scheduler](scheduler.md)

Background automation engine that runs macros when conditions are met. Supports time-based and power-aware triggers.

### [Activity Log](activity.md)

Timestamped history of operations, macro executions, and health changes. Stored as a ring buffer with configurable retention.

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant GUI as Gitty GUI/CLI
    participant Core as gitty-core
    participant Git as Git Remotes
    participant Disk as Config/Logs

    User->>GUI: Add Scan Root
    GUI->>Core: Scan directory
    Core->>Disk: Register repositories
    Core-->>GUI: Repository list

    User->>GUI: Run Macro
    GUI->>Core: Execute steps
    Core->>Git: Git operations
    Core->>Disk: Log activity
    Core-->>GUI: Results

    Scheduler->>Core: Scheduled trigger
    Core->>Git: Background fetch
    Core->>Disk: Update health
```

## Next Steps

- [Domain Model](domain.md) — Detailed terminology
- [Repository Management](repository.md) — UUIDs, re-linking, and identity
- [Organization](organization.md) — Groups and Tags in depth
- [Health System](health.md) — Health checks and scoring
- [Macros](macros.md) — Automation and scripting
- [Scheduler](scheduler.md) — Background automation