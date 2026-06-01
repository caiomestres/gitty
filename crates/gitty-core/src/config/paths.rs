use std::path::PathBuf;

use crate::error::{CoreError, Result};

/// Environment override for the config directory. Useful for tests and for
/// users who want a portable/explicit location.
pub const CONFIG_DIR_ENV: &str = "GITTY_CONFIG_DIR";

/// The Gitty config directory: `$GITTY_CONFIG_DIR` if set, else
/// `dirs::config_dir()/gitty` (`%APPDATA%\gitty` on Windows,
/// `~/.config/gitty` on Linux, `~/Library/Application Support/gitty` on macOS).
pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
        return Ok(PathBuf::from(dir));
    }
    let base = dirs::config_dir().ok_or(CoreError::NoConfigDir)?;
    Ok(base.join("gitty"))
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// Directory holding per-Repository lock files (used in slice 3, ADR-0006).
pub fn locks_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("locks"))
}
