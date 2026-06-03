use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::Result;
use crate::health::WorkspaceHealth;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub severity: Severity,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub read: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTrigger {
    #[default]
    OnCritical,
    OnAnyChange,
    OnSchedulerComplete,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub trigger: NotificationTrigger,
    #[serde(default)]
    pub polling_interval_minutes: Option<u32>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            trigger: NotificationTrigger::default(),
            polling_interval_minutes: Some(5),
        }
    }
}

// ---------------------------------------------------------------------------
// Notification Generation
// ---------------------------------------------------------------------------

/// Generate a health notification by comparing previous and current workspace health.
/// Returns `None` if no notification should be emitted per the trigger configuration.
pub fn generate_health_notification(
    prev: Option<&WorkspaceHealth>,
    current: &WorkspaceHealth,
    trigger: &NotificationTrigger,
) -> Option<Notification> {
    match trigger {
        NotificationTrigger::Disabled => None,

        NotificationTrigger::OnCritical => {
            let prev_critical = prev.map(|p| p.critical_count).unwrap_or(0);
            let new_critical = current.critical_count;

            if new_critical > prev_critical {
                Some(Notification {
                    id: Uuid::new_v4(),
                    timestamp: OffsetDateTime::now_utc(),
                    severity: Severity::Critical,
                    title: "Critical health alert".into(),
                    body: format!(
                        "{} {} critical",
                        new_critical,
                        if new_critical == 1 {
                            "repo is"
                        } else {
                            "repos are"
                        }
                    ),
                    read: false,
                })
            } else {
                None
            }
        }

        NotificationTrigger::OnAnyChange => {
            let changed = match prev {
                None => current.critical_count > 0 || current.warning_count > 0,
                Some(p) => {
                    p.critical_count != current.critical_count
                        || p.warning_count != current.warning_count
                        || p.healthy_count != current.healthy_count
                }
            };

            if changed {
                let severity = if current.critical_count > 0 {
                    Severity::Critical
                } else if current.warning_count > 0 {
                    Severity::Warning
                } else {
                    Severity::Info
                };
                Some(Notification {
                    id: Uuid::new_v4(),
                    timestamp: OffsetDateTime::now_utc(),
                    severity,
                    title: "Health status changed".into(),
                    body: format!(
                        "Score: {:.0}% — {} critical, {} warning, {} healthy",
                        current.score,
                        current.critical_count,
                        current.warning_count,
                        current.healthy_count
                    ),
                    read: false,
                })
            } else {
                None
            }
        }

        NotificationTrigger::OnSchedulerComplete => Some(Notification {
            id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
            severity: if current.critical_count > 0 {
                Severity::Critical
            } else if current.warning_count > 0 {
                Severity::Warning
            } else {
                Severity::Info
            },
            title: "Scheduler run completed".into(),
            body: format!(
                "Score: {:.0}% — {} critical, {} warning, {} healthy",
                current.score, current.critical_count, current.warning_count, current.healthy_count
            ),
            read: false,
        }),
    }
}

/// Remove notifications older than `ttl_days`.
pub fn purge_expired(notifications: &mut Vec<Notification>, ttl_days: u32) {
    let now = OffsetDateTime::now_utc();
    notifications.retain(|n| (now - n.timestamp).whole_days() < ttl_days as i64);
}

// ---------------------------------------------------------------------------
// Notification History Persistence (sidecar file)
// ---------------------------------------------------------------------------

const NOTIFICATIONS_FILE: &str = "notifications.json";

/// Load notification history from the `notifications.json` sidecar in `dir`.
/// Returns an empty vec if the file is missing or corrupt.
pub fn load_history(dir: &Path) -> Vec<Notification> {
    let path = dir.join(NOTIFICATIONS_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Persist notification history to `notifications.json` in `dir`.
/// Uses atomic temp+rename for safe concurrent access.
pub fn save_history(notifications: &[Notification], dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(NOTIFICATIONS_FILE);
    let json = serde_json::to_vec_pretty(notifications)?;
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
    use crate::health::WorkspaceHealth;

    fn make_health(critical: usize, warning: usize, healthy: usize) -> WorkspaceHealth {
        let total = critical + warning + healthy;
        let not_crit = total - critical;
        let score = if total > 0 {
            (not_crit as f64 / total as f64) * 100.0
        } else {
            -1.0
        };
        WorkspaceHealth {
            score,
            total_repos: total,
            critical_count: critical,
            warning_count: warning,
            healthy_count: healthy,
            repositories: vec![],
        }
    }

    #[test]
    fn disabled_trigger_returns_none() {
        let current = make_health(3, 0, 7);
        let result = generate_health_notification(None, &current, &NotificationTrigger::Disabled);
        assert!(result.is_none());
    }

    #[test]
    fn on_critical_fires_on_new_critical() {
        let prev = make_health(0, 2, 8);
        let current = make_health(2, 1, 7);
        let result =
            generate_health_notification(Some(&prev), &current, &NotificationTrigger::OnCritical);
        assert!(result.is_some());
        let n = result.unwrap();
        assert_eq!(n.severity, Severity::Critical);
        assert!(n.body.contains("2 repos are critical"));
    }

    #[test]
    fn on_critical_silent_when_no_new_critical() {
        let prev = make_health(3, 0, 7);
        let current = make_health(2, 1, 7);
        let result =
            generate_health_notification(Some(&prev), &current, &NotificationTrigger::OnCritical);
        assert!(result.is_none());
    }

    #[test]
    fn on_critical_fires_from_zero_prev() {
        let current = make_health(1, 0, 9);
        let result = generate_health_notification(None, &current, &NotificationTrigger::OnCritical);
        assert!(result.is_some());
        assert!(result.unwrap().body.contains("1 repo is critical"));
    }

    #[test]
    fn on_any_change_fires_on_warning_change() {
        let prev = make_health(0, 1, 9);
        let current = make_health(0, 3, 7);
        let result =
            generate_health_notification(Some(&prev), &current, &NotificationTrigger::OnAnyChange);
        assert!(result.is_some());
        assert_eq!(result.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn on_any_change_silent_when_same() {
        let prev = make_health(0, 1, 9);
        let current = make_health(0, 1, 9);
        let result =
            generate_health_notification(Some(&prev), &current, &NotificationTrigger::OnAnyChange);
        assert!(result.is_none());
    }

    #[test]
    fn on_scheduler_complete_always_fires() {
        let current = make_health(0, 0, 10);
        let result =
            generate_health_notification(None, &current, &NotificationTrigger::OnSchedulerComplete);
        assert!(result.is_some());
        assert_eq!(result.unwrap().severity, Severity::Info);
    }

    #[test]
    fn purge_removes_old_notifications() {
        let old_ts = OffsetDateTime::now_utc() - time::Duration::days(10);
        let recent_ts = OffsetDateTime::now_utc();

        let mut notifications = vec![
            Notification {
                id: Uuid::new_v4(),
                timestamp: old_ts,
                severity: Severity::Info,
                title: "old".into(),
                body: "old notification".into(),
                read: true,
            },
            Notification {
                id: Uuid::new_v4(),
                timestamp: recent_ts,
                severity: Severity::Warning,
                title: "recent".into(),
                body: "recent notification".into(),
                read: false,
            },
        ];

        purge_expired(&mut notifications, 7);
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].title, "recent");
    }

    #[test]
    fn purge_with_empty_list() {
        let mut notifications: Vec<Notification> = vec![];
        purge_expired(&mut notifications, 7);
        assert!(notifications.is_empty());
    }

    #[test]
    fn save_and_load_history_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let notifications = vec![Notification {
            id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
            severity: Severity::Info,
            title: "test".into(),
            body: "body".into(),
            read: false,
        }];
        save_history(&notifications, dir.path()).unwrap();
        let loaded = load_history(dir.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "test");
    }

    #[test]
    fn load_history_missing_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_history(dir.path()).is_empty());
    }

    #[test]
    fn load_history_corrupt_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("notifications.json"), b"not json").unwrap();
        assert!(load_history(dir.path()).is_empty());
    }

    #[test]
    fn aggregate_message_uses_singular() {
        let current = make_health(1, 0, 9);
        let result = generate_health_notification(None, &current, &NotificationTrigger::OnCritical);
        assert!(result.unwrap().body.contains("1 repo is"));
    }
}
