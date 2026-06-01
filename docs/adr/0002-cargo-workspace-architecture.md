# Cargo workspace with three crates: core, CLI, Tauri

The Rust backend is structured as a Cargo workspace with three crates: `gitty-core` (pure domain logic — no Tauri dependency), `gitty-cli` (clap-based CLI binary), and `gitty-tauri` (Tauri desktop app). Both `gitty-cli` and `gitty-tauri` depend on `gitty-core`.

This keeps domain logic testable in isolation, lets the CLI ship independently without Tauri dependencies, and avoids coupling release cycles. The CLI and GUI are fully independent processes that share the same Config file, with file-level Locks preventing concurrent operations on the same Repository. No daemon or IPC between them for v1.
