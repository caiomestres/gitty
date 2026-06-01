# JSON for the Config file format

Gitty persists all user configuration — the Workspace definition, the Repository registry, and (later) Macros and Scheduler rules — as a single JSON file resolved via `dirs::config_dir()`. The file carries a schema `version` field that is independent of the application's semantic version.

JSON was chosen over TOML because `serde_json` is already a workspace dependency (zero new deps) and the nested Workspace/registry structures serialize cleanly, whereas TOML becomes awkward for arrays of structured records. The trade-off is reduced hand-editability; this is acceptable because Gitty owns the file and exposes editing through its own UI/CLI.

While Gitty is pre-1.0 (0.x), no migration code is written: if the schema `version` is unrecognised, Gitty fails with a clear error rather than attempting to migrate or silently corrupting data. Backward-compatibility guarantees begin at 1.0.
