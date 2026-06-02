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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
