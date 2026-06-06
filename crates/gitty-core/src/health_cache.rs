use std::path::Path;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::Result;
use crate::health::WorkspaceHealth;

const HEALTH_FILE: &str = "health.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedHealth {
    pub last_evaluated: String,
    pub workspace_health: WorkspaceHealth,
}

/// Save workspace health to `health.json` in the given directory.
/// Uses atomic temp+rename for safe concurrent access.
pub fn save(health: &WorkspaceHealth, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    let path = dir.join(HEALTH_FILE);
    let cached = CachedHealth {
        last_evaluated: OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "unknown".into()),
        workspace_health: health.clone(),
    };

    let json = serde_json::to_vec_pretty(&cached)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)?;

    Ok(())
}

/// Load cached health from `health.json` in the given directory.
/// Returns `None` if the file is missing or corrupt.
pub fn load(dir: &Path) -> Option<CachedHealth> {
    let path = dir.join(HEALTH_FILE);
    let bytes = std::fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::WorkspaceHealth;

    fn sample_health() -> WorkspaceHealth {
        WorkspaceHealth {
            score: Some(80.0),
            total_repos: 10,
            critical_count: 2,
            warning_count: 3,
            healthy_count: 5,
            repositories: vec![],
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let health = sample_health();
        save(&health, dir.path()).unwrap();

        let cached = load(dir.path()).unwrap();
        assert!((cached.workspace_health.score.unwrap() - 80.0).abs() < 0.01);
        assert_eq!(cached.workspace_health.total_repos, 10);
        assert!(!cached.last_evaluated.is_empty());
    }

    #[test]
    fn load_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path()).is_none());
    }

    #[test]
    fn load_corrupt_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(HEALTH_FILE), b"not json").unwrap();
        assert!(load(dir.path()).is_none());
    }

    #[test]
    fn save_creates_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("sub").join("dir");
        save(&sample_health(), &nested).unwrap();
        assert!(load(&nested).is_some());
    }

    #[test]
    fn save_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        save(&sample_health(), dir.path()).unwrap();

        let mut h2 = sample_health();
        h2.score = Some(50.0);
        save(&h2, dir.path()).unwrap();

        let cached = load(dir.path()).unwrap();
        assert!((cached.workspace_health.score.unwrap() - 50.0).abs() < 0.01);
    }

    #[test]
    fn cached_health_has_last_evaluated() {
        let dir = tempfile::tempdir().unwrap();
        save(&sample_health(), dir.path()).unwrap();
        let cached = load(dir.path()).unwrap();
        assert!(OffsetDateTime::parse(&cached.last_evaluated, &Rfc3339).is_ok());
    }
}
