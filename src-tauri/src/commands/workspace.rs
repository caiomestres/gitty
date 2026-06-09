use std::path::{Path, PathBuf};
use std::time::Instant;

use gitty_core::activity::OperationType;
use gitty_core::git::read::{self, ChangeStatus};
use gitty_core::git::write::{BatchOp, GitResult, RepoOutcome};
use gitty_core::repository::RepositoryState;
use gitty_core::scan_and_reconcile;
use gitty_core::ReconcileReport;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::{find_active_repo, find_repo, repo_to_dto, RepoDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFileDto {
    path: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatusDto {
    id: String,
    branch: Option<String>,
    detached: bool,
    dirty: bool,
    ahead: u32,
    behind: u32,
    head_summary: Option<String>,
    head_short_id: Option<String>,
    changed_files_count: usize,
    changed_files: Vec<ChangedFileDto>,
}

fn change_status_label(status: ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Added => "added",
        ChangeStatus::Modified => "modified",
        ChangeStatus::Deleted => "deleted",
        ChangeStatus::Renamed => "renamed",
        ChangeStatus::Untracked => "untracked",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResultDto {
    found: usize,
    new: usize,
    relinked: usize,
    existing: usize,
    missing: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpResultDto {
    success: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResultDto {
    success_count: usize,
    failed_count: usize,
    skipped_count: usize,
    details: Vec<RepoOpDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOpDto {
    repo_name: String,
    repo_path: String,
    success: bool,
    message: String,
}

fn reconcile_to_dto(report: ReconcileReport) -> ScanResultDto {
    ScanResultDto {
        found: report.found,
        new: report.new,
        relinked: report.relinked,
        existing: report.existing,
        missing: report.missing,
    }
}

fn git_result_to_dto(result: GitResult) -> OpResultDto {
    match result {
        GitResult::Success(output) => {
            let message = if output.stdout.trim().is_empty() {
                "Operation completed successfully".into()
            } else {
                output.stdout.trim().to_string()
            };
            OpResultDto {
                success: true,
                message,
            }
        }
        GitResult::Failed { output, category } => OpResultDto {
            success: false,
            message: format!("{category}: {}", output.stderr.trim()),
        },
    }
}

fn batch_to_dto(batch: gitty_core::git::write::BatchResult) -> BulkResultDto {
    let details = batch
        .results
        .iter()
        .map(|r| {
            let (success, message) = match &r.outcome {
                RepoOutcome::Success(output) => {
                    let msg = if output.stdout.trim().is_empty() {
                        "OK".into()
                    } else {
                        output.stdout.trim().to_string()
                    };
                    (true, msg)
                }
                RepoOutcome::Failed { output, category } => {
                    (false, format!("{category}: {}", output.stderr.trim()))
                }
                RepoOutcome::Skipped { reason } => (false, reason.clone()),
                RepoOutcome::Locked { holder_pid, since } => {
                    (false, format!("locked by PID {holder_pid} since {since}"))
                }
            };
            RepoOpDto {
                repo_name: r
                    .repo_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
                repo_path: r.repo_path.display().to_string(),
                success,
                message,
            }
        })
        .collect();

    BulkResultDto {
        success_count: batch.success_count(),
        failed_count: batch.failed_count(),
        skipped_count: batch.skipped_count() + batch.locked_count(),
        details,
    }
}

// ---------------------------------------------------------------------------
// Commands — workspace queries
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_repositories(state: State<'_, AppState>) -> Result<Vec<RepoDto>, AppError> {
    let config = state.config();
    Ok(config
        .workspace
        .repositories
        .iter()
        .map(repo_to_dto)
        .collect())
}

#[tauri::command]
pub fn get_repo_status(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<RepoStatusDto, AppError> {
    let config = state.config();
    let repo = find_repo(&config, &repo_id)?;

    if repo.state == RepositoryState::Missing {
        return Ok(RepoStatusDto {
            id: repo_id,
            branch: None,
            detached: false,
            dirty: false,
            ahead: 0,
            behind: 0,
            head_summary: None,
            head_short_id: None,
            changed_files_count: 0,
            changed_files: vec![],
        });
    }

    let status = read::read_status(&repo.path)?;
    let (ahead, behind) = status
        .upstream
        .as_ref()
        .map(|u| (u.ahead as u32, u.behind as u32))
        .unwrap_or((0, 0));

    let changed_files = status
        .changed_files
        .iter()
        .map(|f| ChangedFileDto {
            path: f.path.clone(),
            status: change_status_label(f.status).to_string(),
        })
        .collect();

    Ok(RepoStatusDto {
        id: repo_id,
        branch: status.branch,
        detached: status.detached,
        dirty: status.dirty,
        ahead,
        behind,
        head_summary: status.head.as_ref().map(|h| h.subject.clone()),
        head_short_id: status.head.as_ref().map(|h| h.short_id.clone()),
        changed_files_count: status.changed_files.len(),
        changed_files,
    })
}

#[tauri::command]
pub fn list_scan_roots(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let config = state.config();
    Ok(config
        .workspace
        .scan_roots
        .iter()
        .map(|sr| sr.path.display().to_string())
        .collect())
}

// ---------------------------------------------------------------------------
// Commands — workspace mutations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn scan_directory(state: State<'_, AppState>, path: String) -> Result<ScanResultDto, AppError> {
    let start = Instant::now();
    let result = state.with_config_write(|config| {
        let report = scan_and_reconcile(config, Path::new(&path))?;
        Ok(reconcile_to_dto(report))
    });
    let elapsed = start.elapsed().as_millis() as u64;
    let (details, error) = match &result {
        Ok(dto) => (Some(format!("{} found, {} new", dto.found, dto.new)), None),
        Err(e) => (None, Some(e.to_string())),
    };
    state.record_activity(
        OperationType::Scan,
        Some(path),
        details,
        Some(elapsed),
        error,
    );
    result
}

#[tauri::command]
pub fn remove_scan_root(state: State<'_, AppState>, path: String) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let before = config.workspace.scan_roots.len();
        config
            .workspace
            .scan_roots
            .retain(|sr| sr.path != path && sr.path != canonical);
        if config.workspace.scan_roots.len() == before {
            return Err(AppError::new(
                "not_found",
                format!("scan root not found: {path}"),
            ));
        }
        Ok(())
    })
}

#[tauri::command]
pub fn unregister_repository(state: State<'_, AppState>, repo_id: String) -> Result<(), AppError> {
    let uuid = super::parse_uuid(&repo_id)?;
    let name = {
        let config = state.config();
        find_active_repo(&config, &repo_id)
            .map(|r| r.display_name().to_string())
            .ok()
    };
    state.with_config_write(|config| {
        if !config.workspace.unregister_repository(uuid) {
            return Err(AppError::from(gitty_core::CoreError::RepositoryNotFound(
                uuid,
            )));
        }
        Ok(())
    })?;
    state.record_activity(OperationType::Unregister, name, None, None, None);
    Ok(())
}

// ---------------------------------------------------------------------------
// Commands — git operations (single repo)
// ---------------------------------------------------------------------------

fn run_single_repo_op(
    state: &AppState,
    repo_id: &str,
    op_type: OperationType,
    op: impl FnOnce(&gitty_core::git::write::GitBinary, &Path) -> gitty_core::Result<GitResult>,
) -> Result<OpResultDto, AppError> {
    let git = state.git()?;
    let config = state.config();
    let repo = find_active_repo(&config, repo_id)?;
    let name = repo.display_name().to_string();
    let path = repo.path.clone();
    drop(config);
    let start = Instant::now();
    let result = op(&git, &path)?;
    let elapsed = start.elapsed().as_millis() as u64;
    let dto = git_result_to_dto(result);
    state.record_activity(
        op_type,
        Some(name),
        None,
        Some(elapsed),
        if dto.success {
            None
        } else {
            Some(dto.message.clone())
        },
    );
    Ok(dto)
}

#[tauri::command]
pub fn fetch_repo(state: State<'_, AppState>, repo_id: String) -> Result<OpResultDto, AppError> {
    run_single_repo_op(&state, &repo_id, OperationType::Fetch, |git, path| {
        git.fetch(path)
    })
}

#[tauri::command]
pub fn pull_repo(state: State<'_, AppState>, repo_id: String) -> Result<OpResultDto, AppError> {
    run_single_repo_op(&state, &repo_id, OperationType::Pull, |git, path| {
        git.pull(path)
    })
}

#[tauri::command]
pub fn checkout_repo(
    state: State<'_, AppState>,
    repo_id: String,
    branch: String,
) -> Result<OpResultDto, AppError> {
    run_single_repo_op(&state, &repo_id, OperationType::Checkout, |git, path| {
        git.checkout(path, &branch)
    })
}

// ---------------------------------------------------------------------------
// Commands — bulk operations
// ---------------------------------------------------------------------------

fn run_bulk_op(
    state: &AppState,
    op_type: OperationType,
    batch_op: &BatchOp,
) -> Result<BulkResultDto, AppError> {
    let git = state.git()?;
    let config = state.config();
    let repos = config.workspace.repositories.clone();
    drop(config);
    let start = Instant::now();
    let batch = git.run_batch_locked(&repos, batch_op)?;
    let elapsed = start.elapsed().as_millis() as u64;
    let dto = batch_to_dto(batch);
    let label = match op_type {
        OperationType::Fetch => "Fetch",
        OperationType::Pull => "Pull",
        _ => "Bulk",
    };
    state.record_activity(
        op_type,
        None,
        Some(format!(
            "{label} all: {} succeeded, {} failed",
            dto.success_count, dto.failed_count
        )),
        Some(elapsed),
        None,
    );
    Ok(dto)
}

#[tauri::command]
pub fn fetch_all(state: State<'_, AppState>) -> Result<BulkResultDto, AppError> {
    run_bulk_op(&state, OperationType::Fetch, &BatchOp::Fetch)
}

#[tauri::command]
pub fn pull_all(state: State<'_, AppState>) -> Result<BulkResultDto, AppError> {
    run_bulk_op(&state, OperationType::Pull, &BatchOp::Pull)
}

// ---------------------------------------------------------------------------
// Commands — search
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn search_repositories(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResultDto>, AppError> {
    let config = state.config();
    let q = query.to_lowercase();
    let mut results = Vec::new();

    for repo in &config.workspace.repositories {
        let name_match = repo.display_name().to_lowercase().contains(&q);
        let path_match = repo.path.display().to_string().to_lowercase().contains(&q);
        let group_match = repo
            .group_id
            .and_then(|gid| config.workspace.groups.iter().find(|g| g.id == gid))
            .map(|g| g.name.to_lowercase().contains(&q))
            .unwrap_or(false);
        let tag_match = repo.tags.iter().any(|t| t.to_lowercase().contains(&q));

        if name_match || path_match || group_match || tag_match {
            results.push(SearchResultDto {
                id: repo.id.to_string(),
                name: repo.display_name().to_string(),
                path: repo.path.display().to_string(),
            });
        }
    }

    Ok(results)
}
