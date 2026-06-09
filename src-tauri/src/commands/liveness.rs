use gitty_core::liveness::{self, Environment, LivenessCache, LivenessResult, LivenessStatus};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::parse_uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDto {
    pub name: String,
    pub url: String,
    pub health_path: String,
    pub enabled: bool,
    pub interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResultDto {
    pub environment_name: String,
    pub status: String,
    pub checked_at: String,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

fn env_to_dto(env: &Environment) -> EnvironmentDto {
    EnvironmentDto {
        name: env.name.clone(),
        url: env.url.clone(),
        health_path: env.health_path.clone(),
        enabled: env.enabled,
        interval_seconds: env.interval_seconds,
    }
}

fn dto_to_env(dto: &EnvironmentDto) -> Environment {
    Environment {
        name: dto.name.trim().to_string(),
        url: dto.url.trim().to_string(),
        health_path: if dto.health_path.trim().is_empty() {
            "/health".to_string()
        } else {
            dto.health_path.trim().to_string()
        },
        enabled: dto.enabled,
        interval_seconds: if dto.interval_seconds == 0 {
            300
        } else {
            dto.interval_seconds
        },
    }
}

fn result_to_dto(r: &LivenessResult) -> LivenessResultDto {
    LivenessResultDto {
        environment_name: r.environment_name.clone(),
        status: match r.status {
            LivenessStatus::Up => "up".into(),
            LivenessStatus::Down => "down".into(),
        },
        checked_at: r
            .checked_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        response_time_ms: r.response_time_ms,
        error: r.error.clone(),
    }
}

#[tauri::command]
pub fn list_environments(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Vec<EnvironmentDto>, AppError> {
    let config = state.config();
    let repo = super::find_repo(&config, &repo_id)?;
    Ok(repo.environments.iter().map(env_to_dto).collect())
}

#[tauri::command]
pub fn add_environment(
    state: State<'_, AppState>,
    repo_id: String,
    env: EnvironmentDto,
) -> Result<EnvironmentDto, AppError> {
    let new_env = dto_to_env(&env);
    let errors = liveness::validate_environment(&new_env);
    if !errors.is_empty() {
        return Err(AppError::new("validation_error", errors.join("; ")));
    }

    let uuid = parse_uuid(&repo_id)?;
    state.with_config_write(|config| {
        let repo = config
            .workspace
            .find_repo_mut(uuid)
            .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))?;

        if repo.environments.iter().any(|e| e.name == new_env.name) {
            return Err(AppError::new(
                "duplicate_environment",
                format!("environment '{}' already exists", new_env.name),
            ));
        }

        repo.environments.push(new_env.clone());
        Ok(env_to_dto(&new_env))
    })
}

#[tauri::command]
pub fn update_environment(
    state: State<'_, AppState>,
    repo_id: String,
    env_name: String,
    env: EnvironmentDto,
) -> Result<EnvironmentDto, AppError> {
    let updated = dto_to_env(&env);
    let errors = liveness::validate_environment(&updated);
    if !errors.is_empty() {
        return Err(AppError::new("validation_error", errors.join("; ")));
    }

    let uuid = parse_uuid(&repo_id)?;
    state.with_config_write(|config| {
        let repo = config
            .workspace
            .find_repo_mut(uuid)
            .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))?;

        let existing = repo
            .environments
            .iter_mut()
            .find(|e| e.name == env_name)
            .ok_or_else(|| {
                AppError::new(
                    "environment_not_found",
                    format!("environment '{env_name}' not found"),
                )
            })?;

        *existing = updated.clone();
        Ok(env_to_dto(&updated))
    })
}

#[tauri::command]
pub fn remove_environment(
    state: State<'_, AppState>,
    repo_id: String,
    env_name: String,
) -> Result<(), AppError> {
    let uuid = parse_uuid(&repo_id)?;
    state.with_config_write(|config| {
        let repo = config
            .workspace
            .find_repo_mut(uuid)
            .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))?;

        let before = repo.environments.len();
        repo.environments.retain(|e| e.name != env_name);
        if repo.environments.len() == before {
            return Err(AppError::new(
                "environment_not_found",
                format!("environment '{env_name}' not found"),
            ));
        }
        Ok(())
    })
}

#[tauri::command]
pub fn probe_environment_cmd(
    state: State<'_, AppState>,
    liveness_cache: State<'_, std::sync::Mutex<LivenessCache>>,
    repo_id: String,
    env_name: String,
) -> Result<LivenessResultDto, AppError> {
    let uuid = parse_uuid(&repo_id)?;

    let env = {
        let config = state.config();
        let repo = super::find_repo(&config, &repo_id)?;
        repo.environments
            .iter()
            .find(|e| e.name == env_name)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    "environment_not_found",
                    format!("environment '{env_name}' not found"),
                )
            })?
    };

    let result = liveness::probe_environment(&env, liveness::reqwest_http_get);

    let dto = result_to_dto(&result);
    let mut cache = liveness_cache
        .lock()
        .expect("liveness cache mutex poisoned");
    cache.store(uuid, result);

    Ok(dto)
}

#[tauri::command]
pub fn get_liveness_results(
    state: State<'_, AppState>,
    liveness_cache: State<'_, std::sync::Mutex<LivenessCache>>,
    repo_id: String,
) -> Result<Vec<LivenessResultDto>, AppError> {
    let uuid = parse_uuid(&repo_id)?;
    let _repo = super::find_repo(&state.config(), &repo_id)?;

    let cache = liveness_cache
        .lock()
        .expect("liveness cache mutex poisoned");
    let results = cache.get_all_for_repo(uuid);
    Ok(results.into_iter().map(result_to_dto).collect())
}

#[tauri::command]
pub fn get_all_liveness_results(
    liveness_cache: State<'_, std::sync::Mutex<LivenessCache>>,
) -> Result<Vec<RepoLivenessDto>, AppError> {
    let cache = liveness_cache
        .lock()
        .expect("liveness cache mutex poisoned");
    let mut map: std::collections::HashMap<uuid::Uuid, Vec<LivenessResultDto>> =
        std::collections::HashMap::new();

    for ((repo_id, _), result) in &cache.results {
        map.entry(*repo_id).or_default().push(result_to_dto(result));
    }

    Ok(map
        .into_iter()
        .map(|(repo_id, results)| RepoLivenessDto {
            repo_id: repo_id.to_string(),
            results,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLivenessDto {
    pub repo_id: String,
    pub results: Vec<LivenessResultDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLivenessDot {
    pub name: String,
    /// "up" | "down" | "gray"
    pub status: String,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDashboardLiveness {
    pub repo_id: String,
    pub dots: Vec<DashboardLivenessDot>,
}

#[tauri::command]
pub fn get_dashboard_liveness(
    state: State<'_, AppState>,
    liveness_cache: State<'_, std::sync::Mutex<LivenessCache>>,
) -> Result<Vec<RepoDashboardLiveness>, AppError> {
    let config = state.config();
    let cache = liveness_cache
        .lock()
        .expect("liveness cache mutex poisoned");

    let result: Vec<_> = config
        .workspace
        .repositories
        .iter()
        .filter(|r| !r.environments.is_empty())
        .map(|repo| {
            let dots = repo
                .environments
                .iter()
                .map(|env| match cache.get(repo.id, &env.name) {
                    Some(lr) => DashboardLivenessDot {
                        name: env.name.clone(),
                        status: match lr.status {
                            LivenessStatus::Up => "up".into(),
                            LivenessStatus::Down => "down".into(),
                        },
                        response_time_ms: lr.response_time_ms,
                    },
                    None => DashboardLivenessDot {
                        name: env.name.clone(),
                        status: "gray".into(),
                        response_time_ms: None,
                    },
                })
                .collect();
            RepoDashboardLiveness {
                repo_id: repo.id.to_string(),
                dots,
            }
        })
        .collect();

    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSuggestionDto {
    pub name: String,
    pub url: String,
    pub health_path: String,
    pub source_file: String,
    pub description: String,
}

#[tauri::command]
pub fn discover_endpoints(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Vec<EndpointSuggestionDto>, AppError> {
    let uuid = parse_uuid(&repo_id)?;
    let config = state.config();
    let repo = config
        .workspace
        .find_by_id(uuid)
        .ok_or_else(|| AppError::from(gitty_core::CoreError::RepositoryNotFound(uuid)))?;

    let suggestions = liveness::discover_endpoints(&repo.path);
    Ok(suggestions
        .into_iter()
        .map(|s| EndpointSuggestionDto {
            name: s.name,
            url: s.url,
            health_path: s.health_path,
            source_file: s.source_file,
            description: s.description,
        })
        .collect())
}
