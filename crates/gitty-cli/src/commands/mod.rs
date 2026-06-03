pub mod filter;
pub mod group;
pub mod health;
pub mod macros;
pub mod notification;
pub mod scheduler;
pub mod tag;
pub mod workspace;

use anyhow::Result;
use gitty_core::config::Config;
use uuid::Uuid;

pub fn resolve_group_id(config: &Config, name_or_id: &str) -> Result<Uuid> {
    if let Ok(id) = Uuid::parse_str(name_or_id) {
        if config.workspace.list_groups().iter().any(|g| g.id == id) {
            return Ok(id);
        }
    }
    config
        .workspace
        .list_groups()
        .iter()
        .find(|g| g.name == name_or_id)
        .map(|g| g.id)
        .ok_or_else(|| anyhow::anyhow!("no group matching '{name_or_id}'"))
}
