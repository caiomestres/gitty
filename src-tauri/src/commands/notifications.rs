use gitty_core::notification::{self, NotificationConfig, Severity};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::format_description::well_known::Rfc3339;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDto {
    pub id: String,
    pub timestamp: String,
    pub severity: Severity,
    pub title: String,
    pub body: String,
    pub read: bool,
}

impl From<&gitty_core::Notification> for NotificationDto {
    fn from(n: &gitty_core::Notification) -> Self {
        Self {
            id: n.id.to_string(),
            timestamp: n.timestamp.format(&Rfc3339).unwrap_or_default(),
            severity: n.severity,
            title: n.title.clone(),
            body: n.body.clone(),
            read: n.read,
        }
    }
}

#[tauri::command]
pub fn get_notifications(state: State<'_, AppState>) -> Result<Vec<NotificationDto>, AppError> {
    let config_dir = state.config_dir()?;
    let mut history = notification::load_history(&config_dir);
    notification::purge_expired(&mut history, 7);
    let _ = notification::save_history(&history, &config_dir);
    Ok(history.iter().map(NotificationDto::from).collect())
}

#[tauri::command]
pub fn mark_notification_read(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let uuid = super::parse_uuid(&id)?;
    let config_dir = state.config_dir()?;
    let mut history = notification::load_history(&config_dir);
    if let Some(notif) = history.iter_mut().find(|n| n.id == uuid) {
        notif.read = true;
        let _ = notification::save_history(&history, &config_dir);
    }
    Ok(())
}

#[tauri::command]
pub fn get_notification_config(state: State<'_, AppState>) -> Result<NotificationConfig, AppError> {
    let config = state.config();
    Ok(config.notifications.clone())
}

#[tauri::command]
pub fn set_notification_config(
    state: State<'_, AppState>,
    config: NotificationConfig,
) -> Result<(), AppError> {
    state.with_config_write(|cfg| {
        cfg.notifications = config;
        Ok(())
    })
}
