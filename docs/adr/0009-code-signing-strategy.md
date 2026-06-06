# Code Signing Strategy: Platform-Specific, Cost-Aware

Release binaries must be signed to avoid OS trust warnings (Windows SmartScreen, macOS Gatekeeper) that block or discourage installation. The project is OSS with no budget for paid certificates, so the strategy is tiered by platform and cost.

## Considered Options

- **Skip signing entirely (rejected):** Windows Smart App Control blocks unsigned executables outright. macOS Gatekeeper shows "damaged app" on Apple Silicon for unsigned binaries. Unacceptable UX for end users.
- **Paid certificates on all platforms (rejected):** Windows EV certificates cost $200+/yr. Apple Developer is $99/yr. No budget for either at launch.
- **Platform-specific free tiers (chosen):** Windows has SignPath.io (free for OSS). macOS has free ad-hoc codesigning. Both are CI-automatable.

## Decision

### Windows
SignPath.io (free for OSS projects): apply at signpath.io/open-source, integrate into GitHub Actions release workflow. Produces properly signed NSIS installers that pass SmartScreen and Smart App Control. The release workflow submits the installer to SignPath's API, polls for completion, and downloads the signed artifact — all gated on the `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG`, and `SIGNPATH_SIGNING_POLICY_SLUG` secrets being configured.

### macOS
Ad-hoc codesign (`codesign -s -`) in CI. This prevents the "app is damaged" error on Apple Silicon but does not bypass Gatekeeper's "unidentified developer" warning. Mitigated by:
1. Clear first-launch documentation (right-click → Open → Open).
2. Homebrew cask tap (`brew install --cask gitty`) — Homebrew handles the quarantine flag automatically, giving zero-friction installs for Homebrew users.
3. Future-proofed: when budget allows ($99/yr Apple Developer), swap ad-hoc for a real identity in the same CI pipeline without workflow changes.

### Linux
No signing required. AppImage is self-contained; Linux package managers handle trust via repository GPG keys if we add deb/rpm later.

## Consequences

- Windows users get a fully trusted installer from day one.
- macOS users who install via Homebrew get zero friction. Direct DMG users see one Gatekeeper prompt on first launch (documented).
- The CI release workflow is the single source of truth for all signing — no manual steps.
- Upgrading macOS to full notarization later requires only adding credentials to CI secrets, not restructuring the pipeline.
