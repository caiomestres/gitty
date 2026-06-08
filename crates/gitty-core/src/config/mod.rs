pub mod paths;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::liveness::LivenessConfig;
use crate::notification::NotificationConfig;
use crate::repository::Workspace;
use crate::scheduler::SchedulerConfig;

/// On-disk schema version, independent of the application's semantic version
/// (ADR-0004). Bumped only when the persisted shape changes.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The persisted Gitty configuration (a single JSON file, ADR-0004).
/// Notification history is stored in a separate `notifications.json` sidecar
/// file to keep config reads/writes lean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub workspace: Workspace,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub liveness: LivenessConfig,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_activity_log_limit")]
    pub activity_log_limit: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_activity_log_limit() -> u32 {
    1000
}

fn default_page_size() -> u32 {
    25
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            workspace: Workspace::default(),
            scheduler: SchedulerConfig::default(),
            notifications: NotificationConfig::default(),
            liveness: LivenessConfig::default(),
            theme: default_theme(),
            activity_log_limit: default_activity_log_limit(),
            page_size: default_page_size(),
        }
    }
}

/// Used to peek the schema version before attempting a full deserialize, so a
/// future/unknown schema yields a precise error rather than a generic JSON one.
#[derive(Deserialize)]
struct VersionProbe {
    version: u32,
}

impl Config {
    /// Load from the default platform path. A missing file yields a default
    /// (empty) config; it is not written until [`Config::save`] is called.
    pub fn load() -> Result<Self> {
        let path = paths::config_file()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => return Err(e.into()),
        };

        let probe: VersionProbe = serde_json::from_slice(&bytes)?;
        if probe.version != CURRENT_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchema {
                found: probe.version,
                expected: CURRENT_SCHEMA_VERSION,
            });
        }

        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::config_file()?;
        self.save_to(&path)
    }

    /// Write the config as pretty JSON. The write is made durable-ish by
    /// writing to a sibling temp file and renaming over the target.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.version, CURRENT_SCHEMA_VERSION);
        assert!(config.workspace.repositories.is_empty());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.json");

        let mut config = Config::default();
        config.workspace.add_scan_root("/some/root".into());

        config.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded.version, CURRENT_SCHEMA_VERSION);
        assert_eq!(loaded.workspace.scan_roots.len(), 1);
    }

    #[test]
    fn unknown_schema_version_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, br#"{"version": 9999, "workspace": {}}"#).unwrap();

        let err = Config::load_from(&path).unwrap_err();
        assert!(matches!(
            err,
            CoreError::UnsupportedSchema {
                found: 9999,
                expected: CURRENT_SCHEMA_VERSION
            }
        ));
    }

    #[test]
    fn corrupt_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, b"{ not valid json").unwrap();
        assert!(matches!(Config::load_from(&path), Err(CoreError::Json(_))));
    }
}
