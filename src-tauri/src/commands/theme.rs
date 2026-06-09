use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

const VALID_THEMES: &[&str] = &["default", "dark", "world-cup-brasil"];

#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> Result<String, AppError> {
    let config = state.config();
    let theme = &config.theme;
    if VALID_THEMES.contains(&theme.as_str()) {
        Ok(theme.clone())
    } else {
        Ok("default".to_string())
    }
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), AppError> {
    if !VALID_THEMES.contains(&theme.as_str()) {
        return Err(AppError::new(
            "invalid_theme",
            format!(
                "unknown theme '{}'; valid themes: {}",
                theme,
                VALID_THEMES.join(", ")
            ),
        ));
    }
    state.with_config_write(|config| {
        config.theme = theme;
        Ok(())
    })
}
