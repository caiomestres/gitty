mod common;

use std::collections::HashMap;
use std::path::Path;

use gitty_core::git::read;
use gitty_core::health::{self, CheckSeverity, HealthThresholds};
use gitty_core::health_cache;
use gitty_core::notification::{self, NotificationTrigger};
use gitty_core::repository::Repository;
use gitty_core::scheduler::{self, DayOfWeek, SchedulerConfig, SchedulerTrigger, TimeOfDay};
use time::OffsetDateTime;

fn init_test_repo(dir: &Path) {
    let repo = common::init_repo(dir);
    common::commit_file(&repo, "a.txt", "hello", "init");
}

#[test]
fn scheduler_triggers_health_evaluation_and_notification() {
    let dir = tempfile::tempdir().unwrap();
    init_test_repo(dir.path());

    let repo = Repository::new(dir.path().to_path_buf(), Some("fp".into()));
    let repos = vec![repo];

    let mut config = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Simple {
            interval_minutes: 1,
        },
        ..Default::default()
    };

    let now = OffsetDateTime::now_utc();
    assert!(scheduler::should_run(&config, now, false, 100));

    scheduler::record_run(&mut config, now);
    assert!(config.last_run.is_some());
    assert!(config.next_run.is_some());

    let checks = health::default_checks();
    let thresholds = HealthThresholds::default();
    let mut statuses = HashMap::new();
    for repo in &repos {
        if let Ok(s) = read::read_status(&repo.path) {
            statuses.insert(repo.id, s);
        }
    }
    let health = health::evaluate_workspace(&repos, &statuses, &checks, &thresholds);
    assert!(health.score > 0.0);

    let notif = notification::generate_health_notification(
        None,
        &health,
        &NotificationTrigger::OnSchedulerComplete,
    );
    assert!(notif.is_some());
}

#[test]
fn corrupt_health_cache_triggers_fresh_evaluation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("health.json"), b"not valid json").unwrap();

    let loaded = health_cache::load(dir.path());
    assert!(loaded.is_none());

    let health = gitty_core::WorkspaceHealth {
        score: 100.0,
        total_repos: 1,
        critical_count: 0,
        warning_count: 0,
        healthy_count: 1,
        repositories: vec![],
    };
    health_cache::save(&health, dir.path()).unwrap();
    let reloaded = health_cache::load(dir.path()).unwrap();
    assert!((reloaded.workspace_health.score - 100.0).abs() < 0.01);
}

#[test]
fn stale_pid_file_allows_scheduler_start() {
    let dir = tempfile::tempdir().unwrap();

    let stale_pid = serde_json::json!({
        "pid": u32::MAX - 1,
        "started_at": "2020-01-01T00:00:00Z"
    });
    std::fs::write(
        dir.path().join("scheduler.pid"),
        serde_json::to_vec(&stale_pid).unwrap(),
    )
    .unwrap();

    assert!(!gitty_core::scheduler::daemon::is_already_running(
        dir.path()
    ));
    gitty_core::scheduler::daemon::write_pid_file(dir.path()).unwrap();
    assert!(gitty_core::scheduler::daemon::is_already_running(
        dir.path()
    ));

    gitty_core::scheduler::daemon::remove_pid_file(dir.path()).unwrap();
}

#[test]
fn battery_below_threshold_pauses_scheduler() {
    let config = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Simple {
            interval_minutes: 1,
        },
        ..Default::default()
    };

    let now = OffsetDateTime::now_utc();
    assert!(!scheduler::should_run(&config, now, true, 10));
    assert!(scheduler::should_run(&config, now, true, 50));
    assert!(scheduler::should_run(&config, now, false, 5));
}

#[test]
fn empty_repo_health_checks_skip_appropriately() {
    let dir = tempfile::tempdir().unwrap();
    git2::Repository::init(dir.path()).unwrap();

    let repo = Repository::new(dir.path().to_path_buf(), None);
    let status = read::read_status(&repo.path).unwrap();
    let checks = health::default_checks();
    let thresholds = HealthThresholds::default();

    let rh = health::evaluate_repository(
        &repo,
        &status,
        &checks,
        &thresholds,
        OffsetDateTime::now_utc(),
    );
    assert_eq!(rh.worst_severity, CheckSeverity::Healthy);

    for check in &rh.checks {
        if check.check_id == "stale" {
            assert_eq!(check.severity, CheckSeverity::Healthy);
            assert!(check.message.contains("skipped"));
        }
        if check.check_id == "diverged" {
            assert_eq!(check.severity, CheckSeverity::Healthy);
            assert!(check.message.contains("skipped"));
        }
    }
}

#[test]
fn advanced_trigger_respects_day_constraint() {
    let now = OffsetDateTime::now_utc();
    let today = DayOfWeek::from_weekday(now.weekday());

    let config = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Advanced {
            interval_minutes: 1,
            window_start: TimeOfDay::from_hm(0, 0),
            window_end: TimeOfDay::from_hm(23, 59),
            days: vec![today],
        },
        ..Default::default()
    };

    assert!(scheduler::should_run(&config, now, false, 100));

    let config_no_day = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Advanced {
            interval_minutes: 1,
            window_start: TimeOfDay::from_hm(0, 0),
            window_end: TimeOfDay::from_hm(23, 59),
            days: vec![], // no valid days
        },
        ..Default::default()
    };

    assert!(!scheduler::should_run(&config_no_day, now, false, 100));
}

#[test]
fn advanced_trigger_midnight_crossing_window() {
    let now = OffsetDateTime::now_utc();
    let today = DayOfWeek::from_weekday(now.weekday());

    let config = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Advanced {
            interval_minutes: 1,
            window_start: TimeOfDay::from_hm(22, 0),
            window_end: TimeOfDay::from_hm(6, 0),
            days: vec![today],
        },
        ..Default::default()
    };

    let hour = now.hour();
    let in_window = hour >= 22 || hour <= 6;
    assert_eq!(scheduler::should_run(&config, now, false, 100), in_window);
}

#[test]
fn compute_next_run_advanced_returns_valid_slot() {
    let now = OffsetDateTime::now_utc();
    let all_days = vec![
        DayOfWeek::Mon,
        DayOfWeek::Tue,
        DayOfWeek::Wed,
        DayOfWeek::Thu,
        DayOfWeek::Fri,
        DayOfWeek::Sat,
        DayOfWeek::Sun,
    ];

    let config = SchedulerConfig {
        enabled: true,
        trigger: SchedulerTrigger::Advanced {
            interval_minutes: 30,
            window_start: TimeOfDay::from_hm(9, 0),
            window_end: TimeOfDay::from_hm(17, 0),
            days: all_days,
        },
        ..Default::default()
    };

    let next = scheduler::compute_next_run(&config, now);
    assert!(next.is_some(), "should find a valid slot within 7 days");
    let next_time = next.unwrap();
    let next_hour = next_time.hour();
    assert!(
        (9..=17).contains(&next_hour),
        "next run should be within 9:00-17:00 window"
    );
}

#[test]
fn notification_purge_respects_ttl() {
    use gitty_core::notification::{Notification, Severity};
    use uuid::Uuid;

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
            severity: Severity::Critical,
            title: "recent".into(),
            body: "recent notification".into(),
            read: false,
        },
    ];

    notification::purge_expired(&mut notifications, 7);
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].title, "recent");
}

#[test]
fn scheduler_config_serde_round_trip() {
    let now = OffsetDateTime::now_utc();
    let config = SchedulerConfig {
        enabled: true,
        macro_id: Some(uuid::Uuid::new_v4()),
        trigger: SchedulerTrigger::Advanced {
            interval_minutes: 15,
            window_start: TimeOfDay::from_hm(9, 0),
            window_end: TimeOfDay::from_hm(17, 0),
            days: vec![DayOfWeek::Mon, DayOfWeek::Fri],
        },
        power: scheduler::PowerConfig {
            pause_on_battery: true,
            battery_threshold: 25,
        },
        last_run: Some(now),
        next_run: None,
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let parsed: SchedulerConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.enabled, true);
    assert!(parsed.macro_id.is_some());
    assert!(parsed.last_run.is_some());
    assert!(matches!(parsed.trigger, SchedulerTrigger::Advanced { .. }));
}

#[test]
fn battery_state_returns_valid_values() {
    let (on_battery, level) = gitty_core::power::battery_state();
    assert!(level <= 100);
    let _ = on_battery;
}

#[test]
fn display_name_returns_directory_name() {
    let repo = Repository::new("/some/path/my-project".into(), None);
    assert_eq!(repo.display_name(), "my-project");
}
