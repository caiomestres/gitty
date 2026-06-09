# Frequently Asked Questions

## General

### What is Gitty?

Gitty is a workspace synchronization and orchestration tool for developers managing multiple Git repositories. It discovers repos, keeps them up to date, and provides a unified dashboard for health monitoring and change tracking.

### Is Gitty free?

Yes, Gitty is open source and free to use. It's released under the MIT license.

### Do I need an account to use Gitty?

No. Gitty is fully offline and requires no accounts, sign-ups, or cloud services. All data stays on your machine.

## Installation & Compatibility

### What platforms does Gitty support?

- Windows 10 (1903+) — x64 and ARM64
- macOS 11 (Big Sur)+ — Intel and Apple Silicon
- Linux — Ubuntu 20.04+, other distributions with glibc 2.31+

### Is Gitty signed?

- **Windows**: Signed via SignPath.io (free OSS certificate)
- **macOS**: Ad-hoc signed (Gatekeeper workaround documented)
- **Linux**: Notarization not applicable

### Can I run Gitty from the command line only?

Yes. While Gitty includes a desktop GUI, the `gitty-cli` package provides full functionality via command line. Many users use both interchangeably.

## Privacy & Security

### Does Gitty collect any data?

No. Gitty has no telemetry, analytics, or cloud components. The only network calls made are:
- Git operations to your configured remotes
- HTTP probes to liveness endpoints you explicitly configure

Read the full [Privacy Policy](../privacy.md).

### Where is my data stored?

Configuration and metadata are stored at platform-native locations:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\` |
| macOS | `~/Library/Application Support/gitty/` |
| Linux | `~/.config/gitty/` |

### Is my repository content accessed?

Gitty only reads Git metadata (refs, config, status). It never reads file contents except for Git's own files (like `.git/config`).

## Usage

### How does Gitty identify repositories?

Each repository is assigned a UUID based on its root commit hash. If you move a repository, Gitty recognizes it by this fingerprint and preserves your groups, tags, and history.

### Can Gitty work with private repositories?

Yes. Gitty uses your existing Git authentication (SSH keys, Git credential helpers). It doesn't manage credentials itself.

### Does Gitty support submodules?

Currently, submodules are not handled specially — they're treated as independent nested repositories if discovered during scanning.

### What's the difference between Groups and Tags?

| Feature | Groups | Tags |
|---------|--------|------|
| Structure | Hierarchical tree | Flat list |
| Repository membership | Exactly one group | Zero or more tags |
| Use case | Organizational categories | Status labels |
| Example | `work/backend`, `personal` | `favorite`, `needs-review` |

### Can I undo a bulk operation?

Macros support rollback steps that execute if an operation fails. However, successful Git operations (like `fetch`) cannot be undone — this is a Git limitation.

### How do I exclude directories from scanning?

Gitty automatically ignores common non-repository directories (like `node_modules`, `.cargo`, `target`). There's currently no user-configurable ignore list.

## Troubleshooting

### Why isn't my repository showing up?

1. Ensure the repository has a `.git` directory
2. Check that it's within a configured Scan Root
3. Try a manual rescan from Settings
4. Verify the repository isn't bare (Gitty requires a working directory)

### Why did my repository become "Missing"?

The repository path changed and Gitty couldn't automatically re-link it. Try:
1. Moving the repository back to its original location
2. Removing and re-scanning the new location
3. Check the [troubleshooting guide](../advanced/troubleshooting.md) for manual re-linking

### Why is the scheduler not running?

1. Check that the scheduler is enabled in Settings
2. Verify your time window and power settings
3. Check the logs for errors
4. On Windows, ensure Smart App Control isn't blocking the background process

### The GUI won't start. What do I do?

1. Try launching from terminal to see error output
2. Check that your graphics drivers support WebGL
3. Delete the config directory and restart (this resets all settings)
4. File an issue with the error message

### Where are the logs?

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\logs\` |
| macOS | `~/Library/Application Support/gitty/logs/` |
| Linux | `~/.config/gitty/logs/` |

## Development

### Can I contribute to Gitty?

Yes! See the [Development Guide](../advanced/development.md) for setup instructions.

### What technologies does Gitty use?

- **Backend**: Rust (git2, tokio, tauri)
- **Frontend**: Svelte 5 + TypeScript
- **Desktop**: Tauri 2
- **Docs**: MkDocs Material

### How can I report bugs?

File an issue on [GitHub](https://github.com/caiomestres/gitty/issues) with:
- Gitty version (`gitty --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior