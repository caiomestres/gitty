use gitty_core::scheduler::{daemon, SchedulerConfig};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::format_description::well_known::Rfc3339;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatusDto {
    pub running: bool,
    pub pid: Option<u32>,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub enabled: bool,
}

#[tauri::command]
pub fn get_scheduler_config(state: State<'_, AppState>) -> Result<SchedulerConfig, AppError> {
    Ok(state.config().scheduler.clone())
}

#[tauri::command]
pub fn get_scheduler_status(state: State<'_, AppState>) -> Result<SchedulerStatusDto, AppError> {
    let config = state.config();
    let config_dir = gitty_core::config::paths::config_dir()?;
    let daemon_status = daemon::status(&config_dir);

    Ok(SchedulerStatusDto {
        running: daemon_status.running,
        pid: daemon_status.pid,
        last_run: config
            .scheduler
            .last_run
            .map(|t| t.format(&Rfc3339).unwrap_or_default()),
        next_run: config
            .scheduler
            .next_run
            .map(|t| t.format(&Rfc3339).unwrap_or_default()),
        enabled: config.scheduler.enabled,
    })
}

#[tauri::command]
pub fn set_scheduler_config(
    state: State<'_, AppState>,
    config: SchedulerConfig,
) -> Result<(), AppError> {
    state.with_config_write(|cfg| {
        let last_run = cfg.scheduler.last_run;
        let next_run = cfg.scheduler.next_run;
        cfg.scheduler = config;
        cfg.scheduler.last_run = last_run;
        cfg.scheduler.next_run = next_run;
        Ok(())
    })
}
