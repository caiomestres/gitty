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
    config
        .workspace
        .find_group_by_name_or_id(name_or_id)
        .map(|g| g.id)
        .ok_or_else(|| anyhow::anyhow!("no group matching '{name_or_id}'"))
}
