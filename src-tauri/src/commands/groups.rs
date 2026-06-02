use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::{parse_uuid, repo_to_dto, RepoDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDto {
    id: String,
    name: String,
    parent_id: Option<String>,
    repo_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTreeNodeDto {
    group: GroupDto,
    children: Vec<GroupTreeNodeDto>,
    repos: Vec<RepoDto>,
}

fn group_to_dto(
    group: &gitty_core::Group,
    workspace: &gitty_core::repository::Workspace,
) -> GroupDto {
    let repo_count = workspace.filter_by_group(group.id).len();
    GroupDto {
        id: group.id.to_string(),
        name: group.name.clone(),
        parent_id: group.parent_id.map(|id| id.to_string()),
        repo_count,
    }
}

fn find_group_in_list(groups: &[gitty_core::Group], id: uuid::Uuid) -> Option<&gitty_core::Group> {
    groups.iter().find(|g| g.id == id)
}

fn tree_node_to_dto(
    node: &gitty_core::GroupTreeNode,
    workspace: &gitty_core::repository::Workspace,
) -> GroupTreeNodeDto {
    let repos: Vec<RepoDto> = workspace
        .filter_by_group(node.group.id)
        .iter()
        .map(|r| repo_to_dto(r))
        .collect();

    GroupTreeNodeDto {
        group: group_to_dto(&node.group, workspace),
        children: node
            .children
            .iter()
            .map(|c| tree_node_to_dto(c, workspace))
            .collect(),
        repos,
    }
}

#[tauri::command]
pub fn list_groups(state: State<'_, AppState>) -> Result<Vec<GroupDto>, AppError> {
    let config = state.config();
    Ok(config
        .workspace
        .list_groups()
        .iter()
        .map(|g| group_to_dto(g, &config.workspace))
        .collect())
}

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<GroupDto, AppError> {
    state.with_config_write(|config| {
        let parent = parent_id.map(|id| parse_uuid(&id)).transpose()?;
        let id = config.workspace.create_group(&name, parent)?;
        let group = find_group_in_list(config.workspace.list_groups(), id).expect("just created");
        Ok(group_to_dto(group, &config.workspace))
    })
}

#[tauri::command]
pub fn rename_group(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&id)?;
        config.workspace.rename_group(uuid, &new_name)?;
        Ok(())
    })
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&id)?;
        config.workspace.delete_group(uuid)?;
        Ok(())
    })
}

#[tauri::command]
pub fn move_group(
    state: State<'_, AppState>,
    id: String,
    new_parent_id: Option<String>,
) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&id)?;
        let parent = new_parent_id.map(|pid| parse_uuid(&pid)).transpose()?;
        config.workspace.move_group(uuid, parent)?;
        Ok(())
    })
}

#[tauri::command]
pub fn assign_repo_to_group(
    state: State<'_, AppState>,
    repo_id: String,
    group_id: String,
) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let rid = parse_uuid(&repo_id)?;
        let gid = parse_uuid(&group_id)?;
        config.workspace.assign_repo_to_group(rid, gid)?;
        Ok(())
    })
}

#[tauri::command]
pub fn group_tree(state: State<'_, AppState>) -> Result<Vec<GroupTreeNodeDto>, AppError> {
    let config = state.config();
    Ok(config
        .workspace
        .group_tree()
        .iter()
        .map(|n| tree_node_to_dto(n, &config.workspace))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitty_core::repository::{Repository, Workspace};
    use std::path::PathBuf;

    #[test]
    fn group_dto_includes_repo_count() {
        let mut ws = Workspace::default();
        let gid = ws.create_group("Test", None).unwrap();
        let mut repo = Repository::new(PathBuf::from("/a"), None);
        repo.group_id = Some(gid);
        ws.repositories.push(repo);

        let group = find_group_in_list(ws.list_groups(), gid).unwrap();
        let dto = group_to_dto(group, &ws);
        assert_eq!(dto.name, "Test");
        assert_eq!(dto.repo_count, 1);
    }

    #[test]
    fn tree_node_dto_includes_repos_and_children() {
        let mut ws = Workspace::default();
        ws.ensure_ungrouped();
        let parent = ws.create_group("Work", None).unwrap();
        ws.create_group("Client", Some(parent)).unwrap();
        let mut repo = Repository::new(PathBuf::from("/a"), None);
        repo.group_id = Some(parent);
        ws.repositories.push(repo);

        let tree = ws.group_tree();
        let work_node = tree.iter().find(|n| n.group.id == parent).unwrap();
        let dto = tree_node_to_dto(work_node, &ws);
        assert_eq!(dto.repos.len(), 1);
        assert_eq!(dto.children.len(), 1);
    }
}
