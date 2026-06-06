mod commands;
mod error;
mod state;

use gitty_core::config::paths;
use gitty_core::Config;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_path = paths::config_file().expect("failed to resolve config path");
            let config = Config::load().unwrap_or_default();
            let state = AppState::new(config, config_path);
            state.start_watcher(app.handle().clone());
            app.manage(state);

            let config_dir = paths::config_dir().expect("failed to resolve config dir");
            let has_external_daemon =
                gitty_core::scheduler::daemon::is_already_running(&config_dir);

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                const TICK_INTERVAL_SECS: u64 = 30;
                const HEALTH_POLL_TICKS: u32 = 10; // ~5 minutes at 30s ticks

                let mut ticks_since_health_poll: u32 = 0;
                loop {
                    let app_state = app_handle.state::<AppState>();

                    if !has_external_daemon {
                        let _ = app_state.with_config_write(|config| {
                            gitty_core::scheduler::runner::tick_with_config(config, &config_dir);
                            Ok(())
                        });
                    }

                    ticks_since_health_poll += 1;
                    if ticks_since_health_poll >= HEALTH_POLL_TICKS {
                        ticks_since_health_poll = 0;
                        let config = app_state.config();
                        gitty_core::scheduler::runner::evaluate_health(&config, &config_dir);
                    }

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
            commands::macros::delete_macro,
            commands::macros::run_macro,
            commands::health::get_workspace_health,
            commands::health::get_repository_health,
            commands::health::refresh_health,
            commands::changes::get_changes,
            commands::scheduler::get_scheduler_config,
            commands::scheduler::get_scheduler_status,
            commands::scheduler::set_scheduler_config,
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::get_notification_config,
            commands::notifications::set_notification_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
