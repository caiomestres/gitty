use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::parse_uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDto {
    name: String,
    repo_count: usize,
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<TagDto>, AppError> {
    let config = state.config();
    let tag_names = config.workspace.list_all_tags();
    Ok(tag_names
        .into_iter()
        .map(|name| {
            let repo_count = config.workspace.filter_by_tag(&name).len();
            TagDto { name, repo_count }
        })
        .collect())
}

#[tauri::command]
pub fn add_tag(state: State<'_, AppState>, repo_id: String, tag: String) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&repo_id)?;
        config.workspace.add_tag(uuid, &tag)?;
        Ok(())
    })
}

#[tauri::command]
pub fn remove_tag(
    state: State<'_, AppState>,
    repo_id: String,
    tag: String,
) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&repo_id)?;
        config.workspace.remove_tag(uuid, &tag)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitty_core::repository::{Repository, Workspace};
    use std::path::PathBuf;

    #[test]
    fn tag_dto_includes_repo_count() {
        let mut ws = Workspace::default();
        let mut repo = Repository::new(PathBuf::from("/a"), None);
        repo.tags = vec!["deploy".into(), "hot".into()];
        ws.repositories.push(repo);
        let mut repo2 = Repository::new(PathBuf::from("/b"), None);
        repo2.tags = vec!["deploy".into()];
        ws.repositories.push(repo2);

        let tags = ws.list_all_tags();
        let dtos: Vec<TagDto> = tags
            .into_iter()
            .map(|name| {
                let repo_count = ws.filter_by_tag(&name).len();
                TagDto { name, repo_count }
            })
            .collect();

        assert_eq!(dtos.len(), 2);
        let deploy = dtos.iter().find(|t| t.name == "deploy").unwrap();
        assert_eq!(deploy.repo_count, 2);
        let hot = dtos.iter().find(|t| t.name == "hot").unwrap();
        assert_eq!(hot.repo_count, 1);
    }
}
