use gitty_core::activity::{self, OperationType};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::format_description::well_known::Rfc3339;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntryDto {
    pub timestamp: String,
    pub operation: OperationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<&gitty_core::ActivityEntry> for ActivityEntryDto {
    fn from(e: &gitty_core::ActivityEntry) -> Self {
        Self {
            timestamp: e.timestamp.format(&Rfc3339).unwrap_or_default(),
            operation: e.operation,
            target: e.target.clone(),
            details: e.details.clone(),
            duration_ms: e.duration_ms,
            error: e.error.clone(),
        }
    }
}

#[tauri::command]
pub fn get_activity_log(state: State<'_, AppState>) -> Result<Vec<ActivityEntryDto>, AppError> {
    let config_dir = state.config_dir()?;
    let log = activity::load_log(&config_dir);
    Ok(log.entries.iter().map(ActivityEntryDto::from).collect())
}

#[tauri::command]
pub fn clear_activity_log(state: State<'_, AppState>) -> Result<(), AppError> {
    let config_dir = state.config_dir()?;
    let log = gitty_core::ActivityLog::default();
    activity::save_log(&log, &config_dir)?;
    Ok(())
}
