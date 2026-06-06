# M6 Ship — Tasks

## Task Overview

| # | Task | Priority | Depends | Parallel |
|---|------|----------|---------|----------|
| T1 | CI: lint + test workflow | P1 | — | [P] |
| T2 | CI: release build workflow | P1 | — | [P] |
| T3 | Windows code signing (SignPath) | P1 | T2 | — |
| T4 | macOS ad-hoc signing | P1 | T2 | [P] with T3 |
| T5 | MkDocs site scaffold + content | P1 | — | [P] |
| T6 | CLI reference auto-generation script | P1 | — | [P] |
| T7 | Docs deployment workflow | P1 | T5 | — |
| T8 | Release notes (git-cliff) | P2 | T2 | — |
| T9 | Homebrew cask tap | P2 | T2 | — |

---

## T1: CI — Lint + Test Workflow

**What**: Create a GitHub Actions workflow that runs on push/PR to `main`. Runs `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and `npm run check`.

**Where**: `.github/workflows/ci.yml` (new)

**Done when**:
- Workflow triggers on push and PR to `main`
- Matrix: runs on `ubuntu-latest` (Linux is fastest for CI)
- Steps: checkout, install Rust (stable), install Node.js, `npm ci`, cargo test, cargo clippy, cargo fmt --check, npm run check
- Caching: Rust target directory + npm node_modules
- Workflow passes on clean main branch

**Reqs**: CI-01, CI-02

**Gate**: Push a test branch and verify workflow runs green

**Status**: Done

---

## T2: CI — Release Build Workflow

**What**: Create a GitHub Actions workflow that builds platform installers on version tags (`v*`). Uses `tauri-action` for cross-platform builds.

**Where**: `.github/workflows/release.yml` (new)

**Done when**:
- Workflow triggers on tag push matching `v*`
- Matrix: Windows (x64), macOS (x64 + ARM universal), Linux (x64)
- Uses `tauri-apps/tauri-action@v0` for building
- Creates GitHub Release with all artifacts
- Unsigned artifacts (signing added in T3/T4)

**Reqs**: REL-01, REL-02, CI-03

**Gate**: Push test tag; verify release with 4 artifacts

**Status**: Done

---

## T3: Windows Code Signing (SignPath)

**What**: Integrate SignPath.io into the release workflow to sign Windows NSIS installer.

**Where**: `.github/workflows/release.yml` (update Windows job)

**Depends on**: T2

**Done when**:
- SignPath.io OSS application submitted and approved
- GitHub Actions secret `SIGNPATH_API_TOKEN` configured
- Windows job uploads artifact to SignPath, downloads signed version
- Signed installer attached to GitHub Release
- Fork builds without the secret skip signing gracefully

**Reqs**: SIGN-WIN-01 through SIGN-WIN-03

**Note**: SignPath approval is external and may take days. The workflow should be coded to skip signing when credentials are absent so T2 isn't blocked.

**Gate**: Download signed installer; verify no SmartScreen warning

**Status**: Done

---

## T4: macOS Ad-Hoc Signing

**What**: Add ad-hoc codesigning to the macOS build step in the release workflow.

**Where**: `.github/workflows/release.yml` (update macOS job)

**Depends on**: T2

**Done when**:
- macOS build step includes `codesign -s - --deep --force` on the app bundle before DMG creation
- DMG tested on Apple Silicon: no "app is damaged" error
- Gatekeeper shows "unidentified developer" (expected for ad-hoc) — bypassed via right-click → Open

**Reqs**: SIGN-MAC-01 through SIGN-MAC-03

**Gate**: Download DMG on Apple Silicon Mac; verify no "damaged" error

**Status**: Done

---

## T5: MkDocs Site Scaffold + Content

**What**: Set up MkDocs Material in `docs/` with Getting Started, Concepts, CLI Reference, and GUI Guide pages.

**Where**:
- `mkdocs.yml` (new, project root)
- `docs/index.md` (new — Getting Started)
- `docs/concepts.md` (new — Workspace, Repository, Group, Tag, Macro, Health, Scheduler)
- `docs/cli-reference.md` (new — placeholder, auto-generated content added in T6)
- `docs/gui-guide.md` (new — Dashboard, Health, Changes, Groups, Macros, Settings)
- `requirements-docs.txt` (new — `mkdocs-material`)

**Done when**:
- `mkdocs serve` runs locally and shows all 4 pages
- Getting Started covers: install, first scan, dashboard overview, first fetch-all
- Concepts page uses CONTEXT.md vocabulary
- GUI Guide has section per page with description of functionality
- Site uses MkDocs Material theme with colors matching DESIGN.md (cream canvas, Cursor Orange accent)

**Reqs**: DOCS-01, DOCS-02

**Gate**: `mkdocs build` succeeds

**Status**: Done

---

## T6: CLI Reference Auto-Generation Script

**What**: Create a script that runs each `gitty` subcommand with `--help` and formats output into `docs/cli-reference.md`.

**Where**:
- `scripts/generate-cli-reference.sh` (new) or `scripts/generate-cli-reference.py`
- `docs/cli-reference.md` (overwritten by script)

**Done when**:
- Script builds `gitty` binary, then runs `gitty --help` + `gitty <subcommand> --help` for each subcommand
- Output formatted as Markdown with code blocks
- `docs/cli-reference.md` updated with current help text
- Script can run in CI (Linux) and locally

**Reqs**: CLI-REF-01 through CLI-REF-03

**Gate**: Run script; verify CLI reference matches current `--help` output

**Status**: Done

---

## T7: Docs Deployment Workflow

**What**: Create a GitHub Actions workflow that deploys docs to GitHub Pages on push to `main` when `docs/` files change.

**Where**: `.github/workflows/docs.yml` (new)

**Depends on**: T5

**Done when**:
- Workflow triggers on push to `main` when `docs/**` or `mkdocs.yml` change
- Uses `mkdocs gh-deploy` to push to `gh-pages` branch
- GitHub Pages configured to serve from `gh-pages` branch
- Site accessible at `https://<user>.github.io/gitty/`

**Reqs**: DOCS-04

**Gate**: Push docs change; verify site updates

**Status**: Done

---

## T8: Release Notes (git-cliff)

**What**: Add git-cliff to the release workflow to auto-generate changelog from conventional commits.

**Where**: `.github/workflows/release.yml` (update)

**Depends on**: T2

**Done when**:
- `cliff.toml` configuration file in project root (groups: feat, fix, docs, chore, refactor, test)
- Release workflow runs `git-cliff --latest` to generate notes
- GitHub Release body contains the generated changelog
- Commits grouped by type (Features, Bug Fixes, Documentation, etc.)

**Reqs**: NOTES-01 through NOTES-03

**Gate**: Push tag with conventional commits; verify release notes grouping

**Status**: Done

---

## T9: Homebrew Cask Tap

**What**: Create a Homebrew tap repository with a cask formula for Gitty.

**Where**:
- New repository: `caiomestres/homebrew-tap` (or similar)
- Formula: `Casks/gitty.rb`
- Documentation: update Getting Started docs with `brew install --cask caiomestres/tap/gitty`

**Depends on**: T2 (release must exist for DMG URL)

**Done when**:
- Tap repository exists with cask formula pointing to GitHub Release DMG
- `brew tap caiomestres/tap && brew install --cask gitty` installs successfully
- Formula includes: version, sha256, url (to GitHub Release), app name
- Getting Started docs updated with Homebrew install instructions

**Reqs**: BREW-01 through BREW-03

**Gate**: `brew install --cask` succeeds on macOS

**Status**: Done
