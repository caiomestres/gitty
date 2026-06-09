# Getting Started

Gitty is a workspace synchronization and orchestration platform for developers managing large collections of Git repositories. It discovers repos under your scan roots, keeps them up to date, and surfaces health and change information in a unified dashboard.

!!! info "Privacy-first architecture"
    Gitty is fully offline — no accounts, no telemetry, no analytics, no cloud sync. Network calls are only made to your own Git remotes and to liveness endpoints you explicitly configure. Read more on the [Privacy](privacy.md) page.

## Install

Choose the option that fits your platform:

### GitHub Releases

Download the latest installer for your OS from the [GitHub Releases](https://github.com/caiomestres/gitty/releases) page:

| Platform | Artifact |
|----------|----------|
| Windows | `.exe` NSIS installer |
| macOS | `.dmg` disk image |
| Linux | `.AppImage` or `.deb` |

### Homebrew (macOS)

```bash
brew tap caiomestres/tap
brew install --cask gitty
```

Homebrew handles macOS quarantine automatically, so first launch is frictionless.

### Build from source

**Prerequisites:** Rust (stable), Node.js (LTS), Git, and [Tauri platform dependencies](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/caiomestres/gitty.git
cd gitty
npm install
npm run tauri build
```

Installers are written to `src-tauri/target/release/bundle/`.

## First scan

After launching Gitty, add a **Scan Root** — a directory Gitty scans recursively for `.git` folders.

**GUI:** Open **Settings**, add a scan root path, and click **Scan**.

**CLI:**

```bash
gitty scan ~/projects
gitty list
```

Each discovered repository is assigned a stable UUID. Re-scanning after a move preserves identity via content-fingerprint re-linking.

## Dashboard overview

The **Dashboard** is your workspace at a glance:

- **Repository count** — how many repos are registered
- **Health summary** — aggregate workspace health score (percentage of repos not in critical state)
- **Recent activity** — scheduler runs and macro executions
- **Quick actions** — fetch all, open health view, jump to changes

Click any repository card to see branch, dirty state, tracking status, and assigned group/tags.

## First fetch-all

Keep your workspace current with a bulk fetch across every registered repository.

**GUI:** From the Dashboard, click **Fetch All** (or use the Macros panel to run the built-in fetch macro).

**CLI:**

```bash
gitty fetch
gitty status
```

`gitty fetch` contacts all remotes for every repo. Follow with `gitty status` to see which repos are ahead, behind, or dirty.

## Next steps

- Read [Concepts](concepts.md) for domain terminology
- Browse the [CLI Reference](cli-reference.md) for all commands
- See the [GUI Guide](gui-guide.md) for page-by-page walkthroughs
