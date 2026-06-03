use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;

use time::OffsetDateTime;

use crate::config::Config;
use crate::execution::execute_macro;
use crate::git::read;
use crate::git::write::GitBinary;
use crate::health;
use crate::health_cache;
use crate::macro_def::{GitOp, MacroDef, Step, StepKind};
use crate::notification::generate_health_notification;
use crate::power;
use crate::repository::RepositoryState;
use crate::scheduler;

fn default_fetch_macro() -> MacroDef {
    MacroDef {
        id: uuid::Uuid::nil(),
        name: "__scheduler_default".into(),
        steps: vec![Step {
            kind: StepKind::GitOp(GitOp::Fetch),
            condition: None,
            rollback: None,
            confirm: false,
        }],
        variables: Default::default(),
    }
}

/// Run one scheduler tick: check if we should run, execute the macro,
/// re-evaluate health, and generate notifications.
/// Returns true if a run was executed.
pub fn tick(config_dir: &std::path::Path) -> bool {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let now = OffsetDateTime::now_utc();
    let (on_battery, battery_level) = power::battery_state();

    if !scheduler::should_run(&config.scheduler, now, on_battery, battery_level) {
        return false;
    }

    let macro_def = config
        .scheduler
        .macro_id
        .and_then(|id| config.workspace.find_macro_by_id(id).cloned())
        .unwrap_or_else(default_fetch_macro);

    let active_repos: Vec<_> = config
        .workspace
        .repositories
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect();

    if let Ok(git) = GitBinary::resolve() {
        let _ = execute_macro(&macro_def, &active_repos, &git);
    }

    scheduler::record_run(&mut config.scheduler, now);

    let checks = health::default_checks();
    let thresholds = config.workspace.health_thresholds.clone();
    let mut statuses = HashMap::new();
    for repo in &active_repos {
        if let Ok(s) = read::read_status(&repo.path) {
            statuses.insert(repo.id, s);
        }
    }

    let prev_health = health_cache::load(config_dir).map(|c| c.workspace_health);
    let current_health = health::evaluate_workspace(
        &config.workspace.repositories,
        &statuses,
        &checks,
        &thresholds,
    );

    let _ = health_cache::save(&current_health, config_dir);

    if let Some(notif) = generate_health_notification(
        prev_health.as_ref(),
        &current_health,
        &config.notifications.trigger,
    ) {
        config.notification_history.push(notif);
    }

    let _ = config.save();
    true
}

/// Re-evaluate health without running any macro.
pub fn health_poll(config_dir: &std::path::Path) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return,
    };

    let repos: Vec<_> = config
        .workspace
        .repositories
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect();

    let checks = health::default_checks();
    let thresholds = &config.workspace.health_thresholds;
    let mut statuses = HashMap::new();
    for repo in &repos {
        if let Ok(s) = read::read_status(&repo.path) {
            statuses.insert(repo.id, s);
        }
    }

    let current_health = health::evaluate_workspace(
        &config.workspace.repositories,
        &statuses,
        &checks,
        thresholds,
    );
    let _ = health_cache::save(&current_health, config_dir);
}

/// Run the scheduler in a blocking loop. Checks every `poll_seconds`.
/// Stops when `stop` is set to true.
pub fn run_loop(config_dir: &std::path::Path, poll_seconds: u64, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        tick(config_dir);
        std::thread::sleep(StdDuration::from_secs(poll_seconds));
    }
}
