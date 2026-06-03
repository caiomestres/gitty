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

fn to_dto(n: &gitty_core::Notification) -> NotificationDto {
    NotificationDto {
        id: n.id.to_string(),
        timestamp: n.timestamp.format(&Rfc3339).unwrap_or_default(),
        severity: n.severity,
        title: n.title.clone(),
        body: n.body.clone(),
        read: n.read,
    }
}

#[tauri::command]
pub fn get_notifications(state: State<'_, AppState>) -> Result<Vec<NotificationDto>, AppError> {
    state.with_config_write(|config| {
        notification::purge_expired(&mut config.notification_history, 7);
        Ok(config.notification_history.iter().map(to_dto).collect())
    })
}

#[tauri::command]
pub fn mark_notification_read(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let uuid = super::parse_uuid(&id)?;
    state.with_config_write(|config| {
        if let Some(notif) = config
            .notification_history
            .iter_mut()
            .find(|n| n.id == uuid)
        {
            notif.read = true;
        }
        Ok(())
    })
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
