# Quick Start

Get up and running with Gitty in 5 minutes. This guide walks you through your first repository scan and bulk fetch.

## 1. Launch Gitty

### GUI

Open Gitty from your applications menu or desktop shortcut. You'll see the dashboard with an empty workspace.

### CLI

```bash
gitty --help
```

## 2. Add a Scan Root

A **Scan Root** is a directory where Gitty recursively searches for Git repositories.

### GUI

1. Click **Settings** in the sidebar
2. Under **Scan Roots**, click **Add**
3. Select a directory (e.g., `~/projects` or `C:\dev`)
4. Click **Scan Now**

### CLI

```bash
# Add and scan a directory
gitty scan ~/projects

# List discovered repositories
gitty list
```

Gitty assigns a stable UUID to each repository. Even if you move a repo later, Gitty will recognize it via content fingerprinting.

## 3. View Your Repositories

The **Dashboard** shows all discovered repositories:

- Repository name and current branch
- Status indicators (clean, dirty, ahead/behind)
- Health score summary

### GUI

Navigate to the **Dashboard** to see repository cards. Click any card for detailed information.

### CLI

```bash
# Show all repositories
gitty list

# Show detailed status
gitty status
```

## 4. Fetch All Repositories

The first bulk operation to try is fetching all remotes across your workspace.

### GUI

Click the **Fetch All** button on the Dashboard. A progress indicator shows the operation status for each repository.

### CLI

```bash
# Fetch all remotes for all repositories
gitty fetch

# Then check status again
gitty status
```

## 5. Organize with Groups

As your workspace grows, organize repositories into **Groups**.

### GUI

1. Go to **Groups** in the sidebar
2. Click **Create Group** and name it (e.g., "work")
3. Navigate to a repository detail page
4. Use the **Group** dropdown to assign it

### CLI

```bash
# Create a group
gitty group create work

# List groups
gitty group list

# Assign a repository to a group
gitty group assign <repo-uuid> work

# View the group tree
gitty group tree
```

## 6. Tag Important Repositories

Use **Tags** for cross-cutting categories like "favorite" or "needs-review".

### GUI

On any repository detail page, add tags in the **Tags** section.

### CLI

```bash
# Add a tag to a repository
gitty tag add <repo-uuid> favorite

# List all tags
gitty tag list
```

## 7. Run Your First Macro

**Macros** are reusable sequences of operations.

### GUI

1. Go to **Macros** in the sidebar
2. Click **Create Macro**
3. Name it "Morning Sync"
4. Add a step: `fetch`
5. Run it against all repositories

### CLI

```bash
# Define a macro
gitty macro define "Morning Sync" fetch

# Run it
gitty macro run "Morning Sync"
```

## Next Steps

- [Health](../concepts/health.md) — Monitor repository health
- [Scheduler](../concepts/scheduler.md) — Automate regular operations
- [GUI Guide](../gui/index.md) — Explore all interface features
- [CLI Reference](../cli/index.md) — Command-line documentation