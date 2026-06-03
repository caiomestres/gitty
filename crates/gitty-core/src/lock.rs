//! Per-Repository PID lock files (ADR-0006).
//!
//! Prevents the CLI and GUI from running conflicting operations on the same
//! Repository simultaneously. Lock files live in
//! `<config_dir>/locks/<repository-uuid>.lock` and contain the owning PID +
//! timestamp. A lock whose PID no longer corresponds to a live process is
//! considered stale and may be reclaimed.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::paths;
use crate::error::{CoreError, Result};

/// Serialized content of a lock file.
#[derive(Debug, Serialize, Deserialize)]
struct LockContent {
    pid: u32,
    timestamp: String,
}

/// RAII guard for a per-Repository lock file. The lock file is removed on drop,
/// ensuring cleanup even on panics or early returns.
#[derive(Debug)]
pub struct RepoLock {
    path: PathBuf,
}

impl RepoLock {
    /// Acquire an exclusive lock for the given Repository using the default
    /// locks directory (`<config_dir>/locks/`).
    pub fn acquire(repo_id: Uuid) -> Result<Self> {
        let dir = paths::locks_dir()?;
        Self::acquire_in(repo_id, &dir)
    }

    /// Acquire an exclusive lock in a caller-specified directory.
    ///
    /// If a lock file already exists:
    /// - **live PID** → `Err(CoreError::LockContention)` (fail-fast).
    /// - **dead PID** → stale lock, silently reclaimed.
    /// - **corrupted file** → treated as stale, overwritten.
    pub fn acquire_in(repo_id: Uuid, locks_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(locks_dir)?;
        let path = locks_dir.join(format!("{repo_id}.lock"));

        if path.exists() {
            match read_lock(&path) {
                Some(content) if is_process_alive(content.pid) => {
                    return Err(CoreError::LockContention {
                        repo_id,
                        pid: content.pid,
                        since: content.timestamp,
                    });
                }
                _ => {} // stale or corrupted — reclaim
            }
        }

        let content = LockContent {
            pid: std::process::id(),
            timestamp: now_rfc3339(),
        };
        std::fs::write(&path, serde_json::to_vec_pretty(&content)?)?;

        Ok(Self { path })
    }

    /// Explicitly release the lock. Equivalent to dropping the guard.
    pub fn release(self) {
        // Drop handles cleanup.
    }
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_lock(path: &Path) -> Option<LockContent> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}

use crate::process::is_process_alive;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_creates_lock_file() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let lock = RepoLock::acquire_in(id, dir.path()).unwrap();
        let expected = dir.path().join(format!("{id}.lock"));
        assert!(expected.exists());
        drop(lock);
    }

    #[test]
    fn drop_removes_lock_file() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let expected = dir.path().join(format!("{id}.lock"));
        {
            let _lock = RepoLock::acquire_in(id, dir.path()).unwrap();
            assert!(expected.exists());
        }
        assert!(!expected.exists());
    }

    #[test]
    fn contention_on_live_pid() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let _held = RepoLock::acquire_in(id, dir.path()).unwrap();

        let err = RepoLock::acquire_in(id, dir.path()).unwrap_err();
        assert!(matches!(err, CoreError::LockContention { .. }));
    }

    #[test]
    fn stale_lock_is_reclaimed() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let path = dir.path().join(format!("{id}.lock"));

        let stale = LockContent {
            pid: u32::MAX - 1,
            timestamp: "2020-01-01T00:00:00Z".into(),
        };
        std::fs::write(&path, serde_json::to_vec(&stale).unwrap()).unwrap();

        let lock = RepoLock::acquire_in(id, dir.path());
        assert!(lock.is_ok(), "stale lock should be reclaimed");
    }

    #[test]
    fn corrupted_lock_file_treated_as_stale() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let path = dir.path().join(format!("{id}.lock"));
        std::fs::write(&path, b"not json").unwrap();

        let lock = RepoLock::acquire_in(id, dir.path());
        assert!(lock.is_ok(), "corrupted lock should be treated as stale");
    }

    #[test]
    fn release_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let lock = RepoLock::acquire_in(id, dir.path()).unwrap();
        lock.release();
        // Second release attempt via another acquire should succeed.
        let _lock2 = RepoLock::acquire_in(id, dir.path()).unwrap();
    }

}
