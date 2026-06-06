# M6 Sub-feature: Ship — Specification

## Problem Statement

Gitty has no CI pipeline, no release packaging, no code signing, and no user documentation. Developers cannot install it without building from source. There is no way for users to learn the CLI commands or GUI workflow without reading code.

## Goals

- [x] Automated CI pipeline: lint + test on every push, release builds on version tags
- [x] Platform installers: Windows NSIS, macOS DMG, Linux AppImage
- [x] Windows code signing via SignPath.io (free for OSS)
- [x] macOS ad-hoc codesigning + Homebrew cask for frictionless install
- [x] Documentation site with getting started guide, CLI reference, and GUI guide
- [x] Auto-generated release notes from conventional commits

## Out of Scope

| Feature | Reason |
| --- | --- |
| macOS Apple Developer notarization ($99/yr) | Using free ad-hoc signing + Homebrew cask (ADR-0009) |
| Linux deb/rpm packages | AppImage is universal; distro packages deferred |
| Nightly builds | Release-only builds for v1 |
| i18n / localized docs | English only (D3) |
| Video tutorials | Written docs only for v1 |

---

## User Stories

### P1: CI Pipeline ⭐ MVP

**User Story**: As a contributor, I want every push to run linting and tests automatically so that broken code is caught before merge.

**Acceptance Criteria**:

1. WHEN a push or pull request targets `main` THEN GitHub Actions SHALL run: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `npm run check`
2. WHEN any check fails THEN the workflow SHALL fail and report which step failed
3. WHEN a version tag (`v*`) is pushed THEN the release workflow SHALL trigger

**Independent Test**: Push a commit with a clippy warning; verify CI fails.

---

### P1: Release Builds ⭐ MVP

**User Story**: As a user, I want to download a pre-built installer for my platform so that I don't need to build from source.

**Acceptance Criteria**:

1. WHEN a version tag is pushed THEN system SHALL build: NSIS installer (Windows x64), DMG (macOS x64 + ARM), AppImage (Linux x64)
2. WHEN builds complete THEN system SHALL create a GitHub Release with all artifacts attached
3. WHEN the release is created THEN system SHALL auto-generate release notes from conventional commits since the last tag

**Independent Test**: Push tag `v0.1.0`; verify GitHub Release appears with 4 artifacts.

---

### P1: Windows Code Signing ⭐ MVP

**User Story**: As a Windows user, I want the installer to be signed so that SmartScreen doesn't block installation.

**Acceptance Criteria**:

1. WHEN the Windows build runs in CI THEN system SHALL sign the NSIS installer via SignPath.io
2. WHEN the signed installer is downloaded THEN Windows SmartScreen SHALL not show "unknown publisher" warning
3. WHEN SignPath credentials are not configured (fork builds) THEN system SHALL skip signing and produce an unsigned artifact

**Independent Test**: Download the Windows installer from a release; run it; verify no SmartScreen warning.

---

### P1: macOS Ad-Hoc Signing ⭐ MVP

**User Story**: As a macOS user, I want the DMG to not show "app is damaged" so that I can install by right-clicking once.

**Acceptance Criteria**:

1. WHEN the macOS build runs in CI THEN system SHALL ad-hoc codesign the app bundle (`codesign -s -`)
2. WHEN the DMG is opened on Apple Silicon THEN macOS SHALL NOT show "app is damaged" error
3. WHEN the app is launched for the first time THEN macOS SHALL show "unidentified developer" warning (bypassed via right-click → Open)

**Independent Test**: Download DMG on Apple Silicon Mac; open; verify no "damaged" error.

---

### P2: Homebrew Cask ⭐ MVP

**User Story**: As a macOS user, I want to install via `brew install --cask gitty` so that I get zero Gatekeeper friction.

**Acceptance Criteria**:

1. WHEN a release is published THEN the Homebrew cask formula SHALL be updated with the new version and SHA256
2. WHEN `brew install --cask gitty` is run THEN system SHALL download, install, and launch without Gatekeeper warnings
3. WHEN the cask is inspected THEN it SHALL reference the GitHub Release DMG URL

**Independent Test**: `brew install --cask gitty`; launch; verify app opens.

---

### P1: Documentation Site ⭐ MVP

**User Story**: As a user, I want to read documentation online so that I can learn Gitty without reading source code.

**Acceptance Criteria**:

1. WHEN the docs site is deployed THEN it SHALL be accessible at `https://<user>.github.io/gitty/`
2. WHEN the docs are built THEN they SHALL include: Getting Started guide, CLI Reference, GUI Guide, Concepts page
3. WHEN the CLI Reference page is loaded THEN it SHALL show auto-generated help text for every subcommand
4. WHEN a push to `main` changes files in `docs/` THEN GitHub Actions SHALL auto-deploy the updated site

**Independent Test**: Visit the docs URL; navigate to CLI Reference; verify all subcommands documented.

---

### P1: Auto-Generated CLI Reference ⭐ MVP

**User Story**: As a developer, I want the CLI reference to stay in sync with the actual commands so that documentation never drifts.

**Acceptance Criteria**:

1. WHEN the docs are built THEN a script SHALL run `gitty --help` and each `gitty <subcommand> --help` to capture current help text
2. WHEN the captured help text is formatted THEN it SHALL be embedded in a Markdown file in `docs/`
3. WHEN a new subcommand is added THEN the next docs build SHALL include it automatically

**Independent Test**: Add a new subcommand; build docs; verify it appears in CLI Reference.

---

### P2: Auto-Generated Release Notes

**User Story**: As a maintainer, I want release notes auto-generated from commit messages so that I don't have to write them manually.

**Acceptance Criteria**:

1. WHEN a version tag is pushed THEN git-cliff SHALL generate a changelog from conventional commits since the last tag
2. WHEN the GitHub Release is created THEN the body SHALL contain the generated changelog
3. WHEN commits use conventional format (`feat:`, `fix:`, `docs:`, etc.) THEN they SHALL be grouped by type in the changelog

**Independent Test**: Make 3 conventional commits (feat, fix, docs); push tag; verify release notes group correctly.

---

## Edge Cases

- WHEN a release build fails on one platform THEN the other platforms SHALL still produce artifacts (matrix continues on error)
- WHEN SignPath.io is unreachable THEN the Windows build SHALL produce an unsigned artifact (not fail)
- WHEN the docs build fails THEN CI SHALL fail the workflow (docs are mandatory)
- WHEN no conventional commits exist since last tag THEN release notes SHALL show "No notable changes"

---

## Requirement Traceability

| Requirement ID | Story | Priority | Status |
| --- | --- | --- | --- |
| CI-01 | CI Pipeline (lint + test) | P1 | Done |
| CI-02 | CI Pipeline (fail reporting) | P1 | Done |
| CI-03 | CI Pipeline (tag trigger) | P1 | Done |
| REL-01 | Release Builds (3 platforms) | P1 | Done |
| REL-02 | Release Builds (GitHub Release) | P1 | Done |
| REL-03 | Release Builds (release notes) | P1 | Done |
| SIGN-WIN-01 | Windows Signing (SignPath) | P1 | Done |
| SIGN-WIN-02 | Windows Signing (SmartScreen) | P1 | Done |
| SIGN-WIN-03 | Windows Signing (skip on fork) | P1 | Done |
| SIGN-MAC-01 | macOS Signing (ad-hoc) | P1 | Done |
| SIGN-MAC-02 | macOS Signing (no damaged error) | P1 | Done |
| SIGN-MAC-03 | macOS Signing (Gatekeeper warning) | P1 | Done |
| BREW-01 | Homebrew Cask (formula update) | P2 | Done |
| BREW-02 | Homebrew Cask (install flow) | P2 | Done |
| BREW-03 | Homebrew Cask (release URL) | P2 | Done |
| DOCS-01 | Docs Site (deployed URL) | P1 | Done |
| DOCS-02 | Docs Site (content sections) | P1 | Done |
| DOCS-03 | Docs Site (CLI reference) | P1 | Done |
| DOCS-04 | Docs Site (auto-deploy) | P1 | Done |
| CLI-REF-01 | CLI Reference (auto-generate) | P1 | Done |
| CLI-REF-02 | CLI Reference (formatted markdown) | P1 | Done |
| CLI-REF-03 | CLI Reference (auto-include new) | P1 | Done |
| NOTES-01 | Release Notes (git-cliff) | P2 | Done |
| NOTES-02 | Release Notes (GitHub Release body) | P2 | Done |
| NOTES-03 | Release Notes (group by type) | P2 | Done |

**Coverage:** 25 requirements, 25 verified

---

## Success Criteria

- [x] Push to main triggers CI; clippy warning fails the build
- [x] Tag `v0.1.0` produces GitHub Release with NSIS + DMG + AppImage
- [x] Windows installer passes SmartScreen
- [x] macOS DMG doesn't show "damaged" on Apple Silicon
- [x] `brew install --cask gitty` installs and launches
- [x] Docs site live at GitHub Pages with CLI reference matching current `--help`
