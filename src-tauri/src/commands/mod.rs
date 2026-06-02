pub mod groups;
pub mod macros;
pub mod tags;
pub mod workspace;

use std::path::Path;

use gitty_core::repository::{Repository, RepositoryState};
use gitty_core::Config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDto {
    pub id: String,
    pub path: String,
    pub name: String,
    pub state: String,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
}

pub fn repo_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string()
}

pub fn repo_to_dto(repo: &Repository) -> RepoDto {
    RepoDto {
        id: repo.id.to_string(),
        path: repo.path.display().to_string(),
        name: repo_name(&repo.path),
        state: match repo.state {
            RepositoryState::Active => "active".into(),
            RepositoryState::Missing => "missing".into(),
        },
        group_id: repo.group_id.map(|id| id.to_string()),
        tags: repo.tags.clone(),
    }
}

pub fn parse_uuid(id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(id).map_err(|e| AppError::from(e.to_string()))
}

pub fn find_repo<'a>(config: &'a Config, id: &str) -> Result<&'a Repository, AppError> {
    let uuid = parse_uuid(id)?;
    config
        .workspace
        .find_by_id(uuid)
        .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))
}
