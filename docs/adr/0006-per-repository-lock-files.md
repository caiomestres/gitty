# Per-Repository PID lock files in the Config directory

Gitty prevents the CLI and GUI from running conflicting operations on the same Repository using one lock file per Repository (`<config_dir>/locks/<repository-uuid>.lock`) containing the owning process PID and a timestamp. A lock whose PID no longer corresponds to a live process is considered stale and may be reclaimed.

A PID lock-file model was chosen over OS advisory file locks (e.g. via `fs4`/`fs2`) because it is simple, introspectable (a human or tool can read who holds a lock and since when), portable across the separate CLI and GUI processes, and needs no cross-platform locking abstraction. On contention Gitty fails fast with a clear error naming the holding PID — it does not block or queue in v1.
