use std::path::PathBuf;

/// Errors produced by the core domain layer.
///
/// Library code returns typed errors; the CLI/GUI boundary maps these to
/// user-facing messages (the CLI wraps them with `anyhow`).
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    /// The on-disk Config declares a schema version this build does not
    /// understand. Pre-1.0 there is no migration path (ADR-0004).
    #[error("unsupported config schema version {found} (this build expects {expected})")]
    UnsupportedSchema { found: u32, expected: u32 },

    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),

    /// The platform config directory could not be resolved.
    #[error("could not resolve a configuration directory for this platform")]
    NoConfigDir,

    /// No `git` binary found on `PATH`.
    #[error("git executable not found on PATH — please install Git and ensure it is in your PATH")]
    GitNotFound,

    /// Another process holds the lock for a Repository (ADR-0006).
    #[error("repository {repo_id} is locked by process {pid} (since {since})")]
    LockContention {
        repo_id: uuid::Uuid,
        pid: u32,
        since: String,
    },
}

pub type Result<T> = std::result::Result<T, CoreError>;
