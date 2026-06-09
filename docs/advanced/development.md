# Development

Building Gitty from source for development or customization.

## Prerequisites

### All Platforms

- **Rust** — Latest stable (install via [rustup.rs](https://rustup.rs))
- **Node.js** — LTS version (18.x or 20.x)
- **Git** — For cloning and Git operations
- **Tauri prerequisites** — Platform-specific (see below)

Verify installations:

```bash
rustc --version      # Should show 1.70+
cargo --version
node --version       # Should show 18.x or 20.x
npm --version
git --version
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install dependencies via Homebrew
brew install openssl@3
```

### Linux

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
```

Fedora:

```bash
sudo dnf install gtk3-devel webkit2gtk3-devel librsvg2-devel
```

Arch:

```bash
sudo pacman -S gtk3 webkit2gtk libappindicator-gtk3 librsvg
```

### Windows

Install via Visual Studio Installer:

- **MSVC** — C++ build tools
- **Windows SDK** — Latest Windows SDK

Or use `cargo-xwin` for cross-compilation.

## Clone Repository

```bash
git clone https://github.com/caiomestres/gitty.git
cd gitty
```

## Build

### Install Dependencies

```bash
npm install
```

This installs frontend dependencies and Tauri CLI.

### Development Build

**Full app (GUI):**

```bash
npm run tauri dev
```

This starts the Vite dev server and Tauri in watch mode. Changes to frontend code hot-reload. Rust changes require restart.

**CLI only:**

```bash
cargo build --package gitty-cli
```

### Production Build

**Full app:**

```bash
npm run tauri build
```

Installers created in:
- **Windows:** `src-tauri/target/release/bundle/nsis/`
- **macOS:** `src-tauri/target/release/bundle/dmg/`
- **Linux:** `src-tauri/target/release/bundle/appimage/`

**CLI only:**

```bash
cargo build --release --package gitty-cli
```

Binary at: `target/release/gitty`

## Project Structure

```
gitty/
├── crates/
│   ├── gitty-core/       # Core Rust library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs
│   │   │   ├── repository.rs
│   │   │   ├── health.rs
│   │   │   ├── liveness.rs
│   │   │   ├── activity.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   └── gitty-cli/        # CLI binary
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── src-tauri/            # Tauri desktop app
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── workspace.rs
│   │   │   ├── repository.rs
│   │   │   ├── health.rs
│   │   │   └── ...
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                  # Frontend (Svelte)
│   ├── lib/
│   │   ├── components/
│   │   ├── styles/
│   │   ├── types/
│   │   └── utils/
│   ├── routes/
│   └── app.html
│
├── docs/                 # MkDocs documentation
├── .specs/              # Feature specifications
└── scripts/             # Build scripts
```

## Development Workflow

### Frontend Development

```bash
# Start frontend only (no Rust)
npm run dev

# Or with Tauri dev (includes hot reload)
npm run tauri dev
```

Frontend at: `http://localhost:5173`

### Rust Development

```bash
# Test core library
cd crates/gitty-core
cargo test

# Test with output
cargo test -- --nocapture

# Test specific module
cargo test health

# Run clippy (linter)
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Tauri Commands

Tauri commands bridge Rust backend to frontend:

```rust
// src-tauri/src/commands/workspace.rs
#[tauri::command]
pub async fn list_repositories(
    state: State<'_, AppState>
) -> Result<Vec<RepositoryDto>, String> {
    // Implementation
}
```

Called from frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

const repos = await invoke('list_repositories');
```

### Adding Commands

1. **Define in Rust:**
   ```rust
   // src-tauri/src/commands/myfeature.rs
   #[tauri::command]
   pub fn my_command(arg: String) -> Result<String, String> {
       Ok(format!("Hello, {}", arg))
   }
   ```

2. **Register in lib.rs:**
   ```rust
   mod commands;
   use commands::myfeature::my_command;

   fn run() {
       tauri::Builder::default()
           .invoke_handler(tauri::generate_handler![
               my_command,
               // ... other commands
           ])
   }
   ```

3. **Call from frontend:**
   ```typescript
   const result = await invoke('my_command', { arg: 'world' });
   ```

## Testing

### Unit Tests (Rust)

```bash
# All tests
cargo test

# Specific crate
cargo test --package gitty-core

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Integration Tests

```bash
# CLI integration tests
cargo test --test cli_integration

# Requires test fixtures
./scripts/setup-test-repos.sh
cargo test --test integration
```

### Frontend Tests

```bash
# Vitest tests
npm run test

# With UI
npm run test:ui
```

### End-to-End Tests

```bash
# Tauri integration tests
cargo test --test tauri_integration
```

## Debugging

### Rust Debugging

```bash
# With logging
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=gitty_core::health=trace cargo run

# To file
RUST_LOG=debug cargo run 2> debug.log
```

### Frontend Debugging

1. Open DevTools in Tauri window: `Ctrl+Shift+I` (or `Cmd+Opt+I` on macOS)
2. Or right-click → Inspect
3. Console shows:
   - Frontend logs
   - Tauri invoke errors
   - Network requests

### VS Code Debugging

`.vscode/launch.json` configuration:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri",
      "cargo": {
        "args": ["build", "--manifest-path=src-tauri/Cargo.toml"]
      }
    }
  ]
}
```

## Code Style

### Rust

- **Formatter:** `rustfmt` (run `cargo fmt`)
- **Linter:** Clippy (run `cargo clippy`)
- **Style:** Standard Rust conventions

### TypeScript/Svelte

- **Formatter:** Prettier (run `npm run format`)
- **Linter:** ESLint (run `npm run lint`)

### Pre-commit

```bash
# Install hook
./scripts/install-hooks.sh

# Or manually before committing
cargo fmt
cargo clippy -- -D warnings
npm run format
npm run lint
```

## Contributing

### Fork and Branch

```bash
# Fork on GitHub, then:
git clone https://github.com/yourusername/gitty.git
cd gitty
git checkout -b feature/my-feature
```

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add dark theme support
fix: resolve scheduler panic on macOS
docs: update README with new features
refactor: simplify health check logic
test: add liveness probe tests
chore: update dependencies
```

### Pull Request

1. Push branch to your fork
2. Create PR against `main`
3. Fill out PR template
4. Ensure CI passes
5. Request review

### CI Checks

PRs must pass:

- [ ] Rust formatting (`cargo fmt --check`)
- [ ] Clippy lints (`cargo clippy`)
- [ ] Rust tests (`cargo test`)
- [ ] Frontend formatting (`npm run format:check`)
- [ ] Frontend lint (`npm run lint`)
- [ ] Type check (`npm run check`)
- [ ] Build succeeds (`npm run tauri build`)

## Documentation

### Build Docs

```bash
# Serve locally
mkdocs serve

# Build
mkdocs build

# Deploy (maintainers only)
mkdocs gh-deploy
```

### Update CLI Reference

```bash
# Regenerate from --help output
./scripts/generate-cli-reference.sh
```

## Release Process

### Version Bump

```bash
# Update version in:
# - Cargo.toml (all crates)
# - package.json
# - src-tauri/tauri.conf.json
# - docs/index.md

# Commit
git commit -m "chore: bump version to x.x.x"
```

### Create Tag

```bash
# Create signed tag
git tag -s vx.x.x -m "Release x.x.x"
git push origin vx.x.x
```

### CI Build

GitHub Actions automatically builds:
- Windows installer (NSIS)
- macOS disk image (DMG)
- Linux AppImage

### Release Notes

Auto-generated from conventional commits via `git-cliff`:

```bash
# Generate
git-cliff --output CHANGELOG.md

# Preview next version
git-cliff --unreleased
```

## Troubleshooting

### Build Failures

**`cargo` not found:**
```bash
# Reinstall Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**`node_modules` missing:**
```bash
rm -rf node_modules
npm install
```

**Tauri build fails:**
```bash
# Clean and rebuild
cargo clean
npm run tauri build
```

**Windows: MSVC not found:**
- Install Visual Studio Build Tools
- Or use `cargo-xwin` for cross-compilation

### Runtime Errors

**Config locked:**
```bash
# Remove stale lock
rm ~/.config/gitty/config.lock
```

**Port in use:**
```bash
# Find and kill process
lsof -i :1420  # or whatever port
kill -9 <PID>
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Documentation](https://v2.tauri.app/)
- [Svelte Documentation](https://svelte.dev/docs)
- [Gitty Architecture](../concepts/index.md)
- [GitHub Issues](https://github.com/caiomestres/gitty/issues)

## License

Gitty is MIT licensed. See [LICENSE](https://github.com/caiomestres/gitty/blob/main/LICENSE) for details.