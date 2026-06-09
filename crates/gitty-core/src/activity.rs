use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::error::Result;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub operation: OperationType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Scan,
    Fetch,
    Pull,
    Checkout,
    Unregister,
    MacroRun,
    SchedulerRun,
    LivenessCheck,
    HealthEvaluation,
    ConfigChange,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scan => write!(f, "Scan"),
            Self::Fetch => write!(f, "Fetch"),
            Self::Pull => write!(f, "Pull"),
            Self::Checkout => write!(f, "Checkout"),
            Self::Unregister => write!(f, "Unregister"),
            Self::MacroRun => write!(f, "Macro Run"),
            Self::SchedulerRun => write!(f, "Scheduler Run"),
            Self::LivenessCheck => write!(f, "Liveness Check"),
            Self::HealthEvaluation => write!(f, "Health Evaluation"),
            Self::ConfigChange => write!(f, "Config Change"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityLog {
    pub entries: Vec<ActivityEntry>,
}

// ---------------------------------------------------------------------------
// ActivityLog Methods
// ---------------------------------------------------------------------------

impl ActivityLog {
    /// Append an entry, evicting the oldest if the log exceeds `limit`.
    pub fn append(&mut self, entry: ActivityEntry, limit: u32) {
        self.entries.push(entry);
        let limit = limit as usize;
        if self.entries.len() > limit {
            let excess = self.entries.len() - limit;
            self.entries.drain(..excess);
        }
    }

    /// Return entries matching all supplied filters (combinatorial AND).
    pub fn filtered(
        &self,
        op_type: Option<OperationType>,
        target: Option<&str>,
        date_from: Option<OffsetDateTime>,
        date_to: Option<OffsetDateTime>,
    ) -> Vec<&ActivityEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(op) = op_type {
                    if e.operation != op {
                        return false;
                    }
                }
                if let Some(t) = target {
                    match &e.target {
                        Some(et) if et.to_lowercase().contains(&t.to_lowercase()) => {}
                        _ => return false,
                    }
                }
                if let Some(from) = date_from {
                    if e.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = date_to {
                    if e.timestamp > to {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Persistence (sidecar file, same pattern as notifications.json)
// ---------------------------------------------------------------------------

const ACTIVITY_FILE: &str = "activity.json";

/// Load the activity log from `activity.json` in `dir`.
/// Returns an empty log if the file is missing or corrupt.
pub fn load_log(dir: &Path) -> ActivityLog {
    let path = dir.join(ACTIVITY_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return ActivityLog::default(),
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Persist the activity log to `activity.json` in `dir`.
/// Uses atomic temp+rename for safe concurrent access.
pub fn save_log(log: &ActivityLog, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(ACTIVITY_FILE);
    let json = serde_json::to_vec_pretty(log)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn make_entry(op: OperationType, target: Option<&str>) -> ActivityEntry {
        ActivityEntry {
            timestamp: OffsetDateTime::now_utc(),
            operation: op,
            target: target.map(String::from),
            details: None,
            duration_ms: None,
            error: None,
        }
    }

    fn make_entry_at(op: OperationType, target: Option<&str>, ts: OffsetDateTime) -> ActivityEntry {
        ActivityEntry {
            timestamp: ts,
            operation: op,
            target: target.map(String::from),
            details: None,
            duration_ms: None,
            error: None,
        }
    }

    #[test]
    fn append_preserves_order() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("repo-a")), 100);
        log.append(make_entry(OperationType::Fetch, Some("repo-b")), 100);
        assert_eq!(log.entries.len(), 2);
        assert_eq!(log.entries[0].operation, OperationType::Scan);
        assert_eq!(log.entries[1].operation, OperationType::Fetch);
    }

    #[test]
    fn ring_buffer_evicts_oldest() {
        let mut log = ActivityLog::default();
        for i in 0..5 {
            log.append(
                make_entry(OperationType::Scan, Some(&format!("repo-{i}"))),
                3,
            );
        }
        assert_eq!(log.entries.len(), 3);
        assert_eq!(log.entries[0].target.as_deref(), Some("repo-2"));
        assert_eq!(log.entries[1].target.as_deref(), Some("repo-3"));
        assert_eq!(log.entries[2].target.as_deref(), Some("repo-4"));
    }

    #[test]
    fn eviction_at_exact_limit() {
        let mut log = ActivityLog::default();
        for i in 0..3 {
            log.append(
                make_entry(OperationType::Fetch, Some(&format!("repo-{i}"))),
                3,
            );
        }
        assert_eq!(log.entries.len(), 3);
        // One more tips it over
        log.append(make_entry(OperationType::Fetch, Some("repo-3")), 3);
        assert_eq!(log.entries.len(), 3);
        assert_eq!(log.entries[0].target.as_deref(), Some("repo-1"));
    }

    #[test]
    fn filter_by_operation_type() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("a")), 100);
        log.append(make_entry(OperationType::Fetch, Some("b")), 100);
        log.append(make_entry(OperationType::Scan, Some("c")), 100);

        let result = log.filtered(Some(OperationType::Scan), None, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|e| e.operation == OperationType::Scan));
    }

    #[test]
    fn filter_by_target() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("my-repo")), 100);
        log.append(make_entry(OperationType::Scan, Some("other")), 100);
        log.append(make_entry(OperationType::Scan, None), 100);

        let result = log.filtered(None, Some("my-repo"), None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target.as_deref(), Some("my-repo"));
    }

    #[test]
    fn filter_by_target_case_insensitive() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("My-Repo")), 100);

        let result = log.filtered(None, Some("my-repo"), None, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_by_date_range() {
        let now = OffsetDateTime::now_utc();
        let yesterday = now - time::Duration::days(1);
        let last_week = now - time::Duration::days(7);

        let mut log = ActivityLog::default();
        log.append(
            make_entry_at(OperationType::Scan, Some("old"), last_week),
            100,
        );
        log.append(
            make_entry_at(OperationType::Scan, Some("recent"), yesterday),
            100,
        );
        log.append(make_entry_at(OperationType::Scan, Some("now"), now), 100);

        let two_days_ago = now - time::Duration::days(2);
        let result = log.filtered(None, None, Some(two_days_ago), None);
        assert_eq!(result.len(), 2);

        let result = log.filtered(None, None, None, Some(two_days_ago));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target.as_deref(), Some("old"));
    }

    #[test]
    fn filter_combinatorial() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("repo-a")), 100);
        log.append(make_entry(OperationType::Fetch, Some("repo-a")), 100);
        log.append(make_entry(OperationType::Scan, Some("repo-b")), 100);

        let result = log.filtered(Some(OperationType::Scan), Some("repo-a"), None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target.as_deref(), Some("repo-a"));
        assert_eq!(result[0].operation, OperationType::Scan);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("a")), 100);

        let result = log.filtered(Some(OperationType::Pull), None, None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let entry = ActivityEntry {
            timestamp: OffsetDateTime::now_utc(),
            operation: OperationType::MacroRun,
            target: Some("my-repo".into()),
            details: Some("ran fetch-all".into()),
            duration_ms: Some(1234),
            error: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ActivityEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operation, OperationType::MacroRun);
        assert_eq!(parsed.target.as_deref(), Some("my-repo"));
        assert_eq!(parsed.duration_ms, Some(1234));
        assert!(parsed.error.is_none());
    }

    #[test]
    fn serde_log_round_trip() {
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Scan, Some("a")), 100);
        log.append(make_entry(OperationType::ConfigChange, None), 100);

        let json = serde_json::to_string_pretty(&log).unwrap();
        let parsed: ActivityLog = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entries.len(), 2);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut log = ActivityLog::default();
        log.append(make_entry(OperationType::Fetch, Some("test-repo")), 100);

        save_log(&log, dir.path()).unwrap();
        let loaded = load_log(dir.path());
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].target.as_deref(), Some("test-repo"));
    }

    #[test]
    fn load_missing_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let log = load_log(dir.path());
        assert!(log.entries.is_empty());
    }

    #[test]
    fn load_corrupt_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("activity.json"), b"not json").unwrap();
        let log = load_log(dir.path());
        assert!(log.entries.is_empty());
    }

    #[test]
    fn operation_type_display() {
        assert_eq!(OperationType::Scan.to_string(), "Scan");
        assert_eq!(OperationType::MacroRun.to_string(), "Macro Run");
        assert_eq!(
            OperationType::HealthEvaluation.to_string(),
            "Health Evaluation"
        );
    }
}
