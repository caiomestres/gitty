# Scheduler: Dual-Host Architecture (GUI + Self-Daemonizing CLI)

The Scheduler needs to run background Macros on triggers (interval, time window, day constraints, power state). Rather than making it GUI-only (excluding headless/server users) or requiring a separate OS service (complex packaging), the Scheduler can be hosted by either the Tauri GUI (as a `tokio::spawn` task) or the CLI (as a self-daemonizing process via `gitty scheduler start`). A file-level lock (PID file in the Config directory) ensures only one Scheduler instance is active at a time, regardless of host.

## Considered Options

- **GUI-only (rejected):** Simple, but excludes headless Linux servers and CI environments where the GUI isn't installed. Users with CLI-only installs would have no automation.
- **Separate OS daemon (rejected):** Proper daemon packaging (systemd unit, launchd plist, Windows Service) adds significant packaging complexity and a new binary. Overkill for v1.
- **CLI foreground process (rejected):** Requires the user to manage backgrounding themselves (`nohup`, `screen`, etc.). Poor UX.

## Consequences

- The CLI binary gains a `scheduler` subcommand with `start`, `stop`, `status`.
- `start` forks/detaches from the terminal (platform-specific: `fork` on Unix, detached `CreateProcess` on Windows), writes a PID file, and enters the scheduling loop.
- `stop` reads the PID file and sends SIGTERM/TerminateProcess.
- The GUI checks the PID file on launch: if a CLI scheduler is already running, it defers to it (no duplicate). If not, it starts its own internal scheduler.
- `health.json` writes from the Scheduler (either host) use the same atomic temp+rename pattern as Config, with an advisory file lock to prevent partial reads during concurrent writes.
