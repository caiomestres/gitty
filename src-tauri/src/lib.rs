mod commands;
mod error;
mod state;

use std::path::Path;

use std::sync::Mutex;

use gitty_core::config::paths;
use gitty_core::liveness::LivenessCache;
use gitty_core::{health, scheduler, Config};
use state::AppState;
use tauri::Manager;

/// Run liveness probes for all enabled environments whose interval has elapsed.
fn liveness_tick(app_state: &AppState, liveness_cache: &Mutex<LivenessCache>) {
    use gitty_core::liveness;
    use gitty_core::repository::RepositoryState;

    let config = app_state.config();
    if !config.liveness.enabled {
        return;
    }

    let repos_to_probe: Vec<_> = config
        .workspace
        .repositories
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .filter(|r| !r.environments.is_empty())
        .map(|r| {
            let envs: Vec<_> = {
                let cache = liveness_cache.lock().expect("liveness cache mutex poisoned");
                r.environments
                    .iter()
                    .filter(|e| e.enabled)
                    .filter(|e| cache.should_probe(r.id, &e.name, e.interval_seconds))
                    .cloned()
                    .collect()
            };
            (r.id, envs)
        })
        .filter(|(_, envs)| !envs.is_empty())
        .collect();

    drop(config);

    for (repo_id, envs) in repos_to_probe {
        for env in &envs {
            let result = liveness::probe_environment(env, liveness::reqwest_http_get);
            let mut cache = liveness_cache.lock().expect("liveness cache mutex poisoned");
            cache.store(repo_id, result);
        }
    }
}

/// Run one scheduler tick without holding the config mutex during git execution.
fn scheduler_tick(app_state: &AppState, config_dir: &Path) {
    use gitty_core::execution::execute_macro;
    use gitty_core::job::JobStatus;
    use gitty_core::power;
    use gitty_core::scheduler::runner::default_fetch_macro;
    use time::OffsetDateTime;

    let now = OffsetDateTime::now_utc();
    let (on_battery, battery_level) = power::battery_state();

    let (macro_def, active_repos) = {
        let config = app_state.config();
        if !scheduler::should_run(&config.scheduler, now, on_battery, battery_level) {
            return;
        }
        let macro_def = config
            .scheduler
            .macro_id
            .and_then(|id| config.workspace.find_macro_by_id(id).cloned())
            .unwrap_or_else(default_fetch_macro);
        let active_repos: Vec<_> = health::active_repos(&config.workspace.repositories)
            .into_iter()
            .cloned()
            .collect();
        (macro_def, active_repos)
    };

    let git = match app_state.git() {
        Ok(g) => g,
        Err(_) => return,
    };

    let repo_refs: Vec<_> = active_repos.iter().collect();
    let jobs = execute_macro(&macro_def, &repo_refs, &git);
    for job in &jobs {
        if let JobStatus::Failed { error } = &job.status {
            eprintln!(
                "scheduler: macro '{}' failed on repo {}: {error}",
                macro_def.name, job.repo_id
            );
        }
    }

    let _ = app_state.with_config_write(|config| {
        scheduler::record_run(&mut config.scheduler, now);
        Ok(())
    });

    let config = app_state.config();
    gitty_core::scheduler::runner::evaluate_health(&config, config_dir);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_path = paths::config_file().expect("failed to resolve config path");
            let config = Config::load().unwrap_or_default();
            let state = AppState::new(config, config_path);
            state.start_watcher(app.handle().clone());
            app.manage(state);
            app.manage(Mutex::new(LivenessCache::default()));

            let config_dir = paths::config_dir().expect("failed to resolve config dir");
            let has_external_daemon =
                gitty_core::scheduler::daemon::is_already_running(&config_dir);

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                const TICK_INTERVAL_SECS: u64 = 30;
                const HEALTH_POLL_TICKS: u32 = 10;

                let mut ticks_since_health_poll: u32 = 0;
                loop {
                    let app_state = app_handle.state::<AppState>();

                    if !has_external_daemon {
                        scheduler_tick(&app_state, &config_dir);
                    }

                    ticks_since_health_poll += 1;
                    if ticks_since_health_poll >= HEALTH_POLL_TICKS {
                        ticks_since_health_poll = 0;
                        let config = app_state.config();
                        gitty_core::scheduler::runner::evaluate_health(&config, &config_dir);
                    }

                    let cache = app_handle.state::<Mutex<LivenessCache>>();
                    liveness_tick(&app_state, &cache);

                    std::thread::sleep(std::time::Duration::from_secs(TICK_INTERVAL_SECS));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::list_repositories,
            commands::workspace::get_repo_status,
            commands::workspace::list_scan_roots,
            commands::workspace::scan_directory,
            commands::workspace::remove_scan_root,
            commands::workspace::unregister_repository,
            commands::workspace::fetch_repo,
            commands::workspace::pull_repo,
            commands::workspace::checkout_repo,
            commands::workspace::fetch_all,
            commands::workspace::pull_all,
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::rename_group,
            commands::groups::delete_group,
            commands::groups::move_group,
            commands::groups::assign_repo_to_group,
            commands::groups::group_tree,
            commands::tags::list_tags,
            commands::tags::add_tag,
            commands::tags::remove_tag,
            commands::macros::list_macros,
            commands::macros::get_macro,
            commands::macros::define_macro,
            commands::macros::update_macro,
            commands::macros::delete_macro,
            commands::macros::run_macro,
            commands::health::get_workspace_health,
            commands::health::get_repository_health,
            commands::health::refresh_health,
            commands::liveness::list_environments,
            commands::liveness::add_environment,
            commands::liveness::update_environment,
            commands::liveness::remove_environment,
            commands::liveness::probe_environment_cmd,
            commands::liveness::get_liveness_results,
            commands::liveness::get_all_liveness_results,
            commands::liveness::get_dashboard_liveness,
            commands::changes::get_changes,
            commands::scheduler::get_scheduler_config,
            commands::scheduler::get_scheduler_status,
            commands::scheduler::set_scheduler_config,
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::get_notification_config,
            commands::notifications::set_notification_config,
            commands::theme::get_theme,
            commands::theme::set_theme,
            commands::activity::get_activity_log,
            commands::activity::clear_activity_log,
            commands::pagination::get_page_size,
            commands::pagination::set_page_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
