# Managed state with file-watcher for config cache invalidation

The Tauri GUI caches the loaded `Config` in a `Mutex<Config>` (Tauri managed state) instead of loading from disk on every IPC call. A `notify`-based file-watcher monitors the config file and reloads the in-memory cache whenever the file changes on disk, so CLI modifications are reflected in the GUI without restart.

Stateless load-per-call (the prior approach, D21) was simple but would become increasingly wasteful as M4 adds ~13 new commands on top of the existing 10. Caching alone would break the shared-config architecture (D17: CLI and GUI are independent processes sharing the same file), because the GUI would miss CLI-initiated changes. The file-watcher resolves this: the `notify` crate watches the config path and triggers a reload into the `Mutex`, keeping both processes in sync without a daemon or IPC channel between them.

## Considered Options

- **Stay stateless (load-per-call):** Simple, no cache coherence issues. Rejected because the M4 expansion adds many more commands, and redundant disk reads become noticeable UX overhead on batch operations.
- **Cache without watcher (accept staleness):** Simplest managed-state path. Rejected because users commonly run CLI commands alongside the GUI and would see stale data without explanation.
- **Cache with watcher (chosen):** Balances performance and coherence. The `notify` crate is mature and cross-platform. Cost: one extra dependency and a background watcher thread.
