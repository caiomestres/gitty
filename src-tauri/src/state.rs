use std::path::PathBuf;
use std::sync::Mutex;

use gitty_core::Config;

use tauri::Manager;

use crate::error::AppError;

pub struct AppState {
    config: Mutex<Config>,
    config_path: PathBuf,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            config: Mutex::new(config),
            config_path,
        }
    }

    pub fn config(&self) -> std::sync::MutexGuard<'_, Config> {
        self.config.lock().expect("config mutex poisoned")
    }

    pub fn reload(&self) -> Result<(), AppError> {
        let new_config = Config::load_from(&self.config_path)?;
        *self.config.lock().expect("config mutex poisoned") = new_config;
        Ok(())
    }

    pub fn with_config_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut Config) -> Result<T, AppError>,
    {
        let mut guard = self.config.lock().expect("config mutex poisoned");
        let result = f(&mut guard)?;
        guard.save_to(&self.config_path)?;
        Ok(result)
    }

    pub fn start_watcher(&self, app_handle: tauri::AppHandle) {
        use notify::{EventKind, RecursiveMode, Watcher};
        use tauri::Emitter;

        let config_path = self.config_path.clone();
        let watch_dir = match config_path.parent() {
            Some(p) => p.to_path_buf(),
            None => return,
        };

        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher = match notify::recommended_watcher(tx) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("failed to create file watcher: {e}");
                    return;
                }
            };

            if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
                eprintln!("failed to watch config directory: {e}");
                return;
            }

            let mut last_reload = std::time::Instant::now();
            let debounce = std::time::Duration::from_millis(500);

            for result in rx {
                let event = match result {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let is_config = event.paths.contains(&config_path);
                let is_write = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_));

                if is_config && is_write && last_reload.elapsed() > debounce {
                    last_reload = std::time::Instant::now();
                    let state = app_handle.state::<AppState>();
                    if state.reload().is_ok() {
                        let _ = app_handle.emit("config-changed", ());
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_holds_config() {
        let config = Config::default();
        let state = AppState::new(config, PathBuf::from("/tmp/test.json"));
        let guard = state.config();
        assert_eq!(guard.version, gitty_core::config::CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn reload_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut config = Config::default();
        config.workspace.add_scan_root(PathBuf::from("/test"));
        config.save_to(&path).unwrap();

        let state = AppState::new(Config::default(), path);
        assert!(state.config().workspace.scan_roots.is_empty());

        state.reload().unwrap();
        assert_eq!(state.config().workspace.scan_roots.len(), 1);
    }

    #[test]
    fn with_config_write_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let state = AppState::new(Config::default(), path.clone());

        state
            .with_config_write(|config| {
                config.workspace.add_scan_root(PathBuf::from("/added"));
                Ok(())
            })
            .unwrap();

        let reloaded = Config::load_from(&path).unwrap();
        assert_eq!(reloaded.workspace.scan_roots.len(), 1);
    }
}
