use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> Result<String, AppError> {
    let config = state.config();
    Ok(config.theme.clone())
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), AppError> {
    state.with_config_write(|config| {
        config.theme = theme;
        Ok(())
    })
}
