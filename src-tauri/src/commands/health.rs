use std::collections::HashMap;

use gitty_core::git::read;
use gitty_core::health::{self, CheckSeverity};
use gitty_core::health_cache;
use gitty_core::repository::RepositoryState;
use serde::{Deserialize, Serialize};
use tauri::State;
use time::OffsetDateTime;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceHealthDto {
    pub score: f64,
    pub total_repos: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub healthy_count: usize,
    pub repositories: Vec<RepositoryHealthDto>,
    pub last_evaluated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealthDto {
    pub repo_id: String,
    pub repo_name: String,
    pub checks: Vec<CheckResultDto>,
    pub worst_severity: CheckSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResultDto {
    pub check_id: String,
    pub severity: CheckSeverity,
    pub message: String,
}

fn to_workspace_dto(
    wh: &gitty_core::WorkspaceHealth,
    last_evaluated: Option<String>,
) -> WorkspaceHealthDto {
    WorkspaceHealthDto {
        score: wh.score,
        total_repos: wh.total_repos,
        critical_count: wh.critical_count,
        warning_count: wh.warning_count,
        healthy_count: wh.healthy_count,
        repositories: wh
            .repositories
            .iter()
            .map(|rh| RepositoryHealthDto {
                repo_id: rh.repo_id.to_string(),
                repo_name: rh.repo_name.clone(),
                checks: rh
                    .checks
                    .iter()
                    .map(|c| CheckResultDto {
                        check_id: c.check_id.clone(),
                        severity: c.severity,
                        message: c.message.clone(),
                    })
                    .collect(),
                worst_severity: rh.worst_severity,
            })
            .collect(),
        last_evaluated,
    }
}

fn evaluate_fresh(state: &AppState) -> Result<WorkspaceHealthDto, AppError> {
    let config = state.config();
    let repos = &config.workspace.repositories;
    let thresholds = &config.workspace.health_thresholds;
    let checks = health::default_checks();

    let active_repos: Vec<_> = repos
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect();

    let mut statuses = HashMap::new();
    for repo in &active_repos {
        if let Ok(s) = read::read_status(&repo.path) {
            statuses.insert(repo.id, s);
        }
    }

    let workspace_health = health::evaluate_workspace(repos, &statuses, &checks, thresholds);

    if let Ok(dir) = gitty_core::config::paths::config_dir() {
        let _ = health_cache::save(&workspace_health, &dir);
    }

    Ok(to_workspace_dto(&workspace_health, None))
}

#[tauri::command]
pub fn get_workspace_health(state: State<'_, AppState>) -> Result<WorkspaceHealthDto, AppError> {
    if let Ok(dir) = gitty_core::config::paths::config_dir() {
        if let Some(cached) = health_cache::load(&dir) {
            return Ok(to_workspace_dto(
                &cached.workspace_health,
                Some(cached.last_evaluated),
            ));
        }
    }
    evaluate_fresh(&state)
}

#[tauri::command]
pub fn get_repository_health(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<RepositoryHealthDto, AppError> {
    let config = state.config();
    let uuid = super::parse_uuid(&repo_id)?;
    let repo = config
        .workspace
        .find_by_id(uuid)
        .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))?;

    if repo.state == RepositoryState::Missing {
        return Ok(RepositoryHealthDto {
            repo_id,
            repo_name: repo.display_name().to_string(),
            checks: vec![],
            worst_severity: CheckSeverity::Healthy,
        });
    }

    let status = read::read_status(&repo.path)?;
    let thresholds = &config.workspace.health_thresholds;
    let checks = health::default_checks();
    let now = OffsetDateTime::now_utc();
    let rh = health::evaluate_repository(repo, &status, &checks, thresholds, now);

    Ok(RepositoryHealthDto {
        repo_id: rh.repo_id.to_string(),
        repo_name: rh.repo_name,
        checks: rh
            .checks
            .iter()
            .map(|c| CheckResultDto {
                check_id: c.check_id.clone(),
                severity: c.severity,
                message: c.message.clone(),
            })
            .collect(),
        worst_severity: rh.worst_severity,
    })
}

#[tauri::command]
pub fn refresh_health(state: State<'_, AppState>) -> Result<WorkspaceHealthDto, AppError> {
    evaluate_fresh(&state)
}
