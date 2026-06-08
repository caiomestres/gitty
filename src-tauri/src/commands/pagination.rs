use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn get_page_size(state: State<'_, AppState>) -> Result<u32, AppError> {
    let config = state.config();
    Ok(config.page_size)
}

#[tauri::command]
pub fn set_page_size(state: State<'_, AppState>, page_size: u32) -> Result<(), AppError> {
    state.with_config_write(|config| {
        config.page_size = page_size;
        Ok(())
    })
}
