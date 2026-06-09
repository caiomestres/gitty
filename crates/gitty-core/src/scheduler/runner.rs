use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;

use time::OffsetDateTime;

use crate::config::Config;
use crate::execution::execute_macro;
use crate::git::write::GitBinary;
use crate::health;
use crate::health_cache;
use crate::job::JobStatus;
use crate::macro_def::{GitOp, MacroDef, Step, StepKind};
use crate::notification::{self, generate_health_notification};
use crate::power;
use crate::scheduler;

pub fn default_fetch_macro() -> MacroDef {
    MacroDef {
        id: uuid::Uuid::nil(),
        name: "__scheduler_default".into(),
        steps: vec![Step {
            kind: StepKind::GitOp(GitOp::Fetch),
            condition: None,
            rollback: None,
            confirm: false,
            retry: None,
        }],
        variables: Default::default(),
    }
}

/// Run one scheduler tick against the provided config.
/// The caller is responsible for loading and persisting `config`.
/// Returns true if a macro run was executed.
pub fn tick_with_config(config: &mut Config, config_dir: &std::path::Path) -> bool {
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

    let active_repos = health::active_repos(&config.workspace.repositories);

    match GitBinary::resolve() {
        Ok(git) => {
            let jobs = execute_macro(&macro_def, &active_repos, &git);
            for job in &jobs {
                if let JobStatus::Failed { error } = &job.status {
                    eprintln!(
                        "scheduler: macro '{}' failed on repo {}: {error}",
                        macro_def.name, job.repo_id
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("scheduler: failed to resolve git binary: {e}");
        }
    }

    scheduler::record_run(&mut config.scheduler, now);

    evaluate_health_with_repos(config, config_dir, &active_repos);
    true
}

/// Standalone convenience: loads config from disk, runs a tick, saves if changed.
/// Used by the CLI daemon where there is no shared state.
pub fn tick(config_dir: &std::path::Path) -> bool {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let ran = tick_with_config(&mut config, config_dir);
    if ran {
        let _ = config.save();
    }
    ran
}

/// Re-evaluate health and generate notifications.
/// Operates on the provided config so callers with shared state can avoid races.
pub fn evaluate_health(config: &Config, config_dir: &std::path::Path) {
    let active_repos = health::active_repos(&config.workspace.repositories);
    evaluate_health_with_repos(config, config_dir, &active_repos);
}

fn evaluate_health_with_repos(
    config: &Config,
    config_dir: &std::path::Path,
    active_repos: &[&crate::repository::Repository],
) {
    let thresholds = config.workspace.health_thresholds.clone();
    let statuses = health::collect_statuses(active_repos);

    let prev_health = health_cache::load(config_dir).map(|c| c.workspace_health);
    let current_health = health::evaluate_workspace(active_repos, &statuses, &thresholds);

    let _ = health_cache::save(&current_health, config_dir);

    if let Some(notif) = generate_health_notification(
        prev_health.as_ref(),
        &current_health,
        &config.notifications.trigger,
    ) {
        notification::append_to_history(notif, config_dir);
    }
}

/// Re-evaluate health without running any macro (standalone, loads from disk).
pub fn health_poll(config_dir: &std::path::Path) {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return,
    };

    evaluate_health(&config, config_dir);
}

/// Run the scheduler in a blocking loop. Checks every `poll_seconds`.
/// Stops when `stop` is set to true.
pub fn run_loop(config_dir: &std::path::Path, poll_seconds: u64, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        tick(config_dir);
        std::thread::sleep(StdDuration::from_secs(poll_seconds));
    }
}
