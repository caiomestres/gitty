# Installation

Gitty is available for Windows, macOS, and Linux. Choose the method that works best for your platform.

## System Requirements

| Platform | Minimum Version | Architecture |
|----------|-----------------|--------------|
| Windows | 10 (1903+) | x64, ARM64 |
| macOS | 11 (Big Sur)+ | Intel, Apple Silicon |
| Linux | Ubuntu 20.04+ / glibc 2.31+ | x64, ARM64 |

## GitHub Releases

The simplest way to install Gitty is downloading a pre-built release:

1. Visit the [GitHub Releases](https://github.com/caiomestres/gitty/releases) page
2. Download the appropriate artifact for your platform
3. Run the installer

| Platform | Download | Notes |
|----------|----------|-------|
| **Windows** | `.exe` (NSIS) | Standard Windows installer |
| **macOS** | `.dmg` | Drag to Applications folder |
| **Linux** | `.AppImage` | Portable, no installation needed |
| **Linux** | `.deb` | For Debian/Ubuntu-based distributions |

### Windows Installation

1. Download `Gitty-x.x.x-windows.exe`
2. Run the installer (Smart App Control may warn about unsigned apps — see [troubleshooting](../advanced/troubleshooting.md))
3. Gitty installs to `%LOCALAPPDATA%\Programs\Gitty\`
4. The installer optionally adds Gitty to your PATH

### macOS Installation

**Homebrew (recommended):**

```bash
brew tap caiomestres/tap
brew install --cask gitty
```

Homebrew handles macOS quarantine automatically, so first launch is frictionless.

**Manual DMG:**

1. Download `Gitty-x.x.x.dmg`
2. Open the DMG and drag Gitty to Applications
3. On first launch, right-click → Open (Gatekeeper workaround)

### Linux Installation

**AppImage (portable):**

1. Download `Gitty-x.x.x.AppImage`
2. Make it executable: `chmod +x Gitty-x.x.x.AppImage`
3. Run: `./Gitty-x.x.x.AppImage`

**Debian/Ubuntu (.deb):**

```bash
sudo dpkg -i gitty_x.x.x_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

## Build from Source

Building from source requires Rust and Node.js:

**Prerequisites:**

- Rust (latest stable) — [rustup.rs](https://rustup.rs)
- Node.js (LTS) — [nodejs.org](https://nodejs.org)
- Git
- Tauri platform dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

**Build:**

```bash
# Clone the repository
git clone https://github.com/caiomestres/gitty.git
cd gitty

# Install dependencies
npm install

# Build the desktop app
npm run tauri build

# Or build just the CLI
cargo build --release --package gitty-cli
```

Installers are written to `src-tauri/target/release/bundle/`.

## Verifying Installation

After installation, verify Gitty is working:

```bash
# Check CLI is available
gitty --version

# Launch the GUI
gitty
```

## Next Steps

- [Quick Start](quickstart.md) — Your first scan and fetch
- [Core Concepts](../concepts/index.md) — Understand the domain model
- [GUI Guide](../gui/index.md) — Learn the desktop interface