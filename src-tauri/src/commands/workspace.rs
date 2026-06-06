use std::path::{Path, PathBuf};

use gitty_core::git::read::{self, ChangeStatus};
use gitty_core::git::write::{BatchOp, GitBinary, GitResult, RepoOutcome};
use gitty_core::repository::RepositoryState;
use gitty_core::scan_and_reconcile;
use gitty_core::ReconcileReport;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::{find_repo, repo_to_dto, RepoDto};

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
    state.with_config_write(|config| {
        let report = scan_and_reconcile(config, Path::new(&path))?;
        Ok(reconcile_to_dto(report))
    })
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
            return Err(AppError::new("not_found", format!("scan root not found: {path}")));
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// Commands — git operations (single repo)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn fetch_repo(state: State<'_, AppState>, repo_id: String) -> Result<OpResultDto, AppError> {
    let config = state.config();
    let repo = find_repo(&config, &repo_id)?;
    if repo.state == RepositoryState::Missing {
        return Err(AppError::new("repository_missing", "repository path not found"));
    }
    let git = GitBinary::resolve()?;
    let result = git.fetch(&repo.path)?;
    Ok(git_result_to_dto(result))
}

#[tauri::command]
pub fn pull_repo(state: State<'_, AppState>, repo_id: String) -> Result<OpResultDto, AppError> {
    let config = state.config();
    let repo = find_repo(&config, &repo_id)?;
    if repo.state == RepositoryState::Missing {
        return Err(AppError::new("repository_missing", "repository path not found"));
    }
    let git = GitBinary::resolve()?;
    let result = git.pull(&repo.path)?;
    Ok(git_result_to_dto(result))
}

#[tauri::command]
pub fn checkout_repo(
    state: State<'_, AppState>,
    repo_id: String,
    branch: String,
) -> Result<OpResultDto, AppError> {
    let config = state.config();
    let repo = find_repo(&config, &repo_id)?;
    if repo.state == RepositoryState::Missing {
        return Err(AppError::new("repository_missing", "repository path not found"));
    }
    let git = GitBinary::resolve()?;
    let result = git.checkout(&repo.path, &branch)?;
    Ok(git_result_to_dto(result))
}

// ---------------------------------------------------------------------------
// Commands — bulk operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn fetch_all(state: State<'_, AppState>) -> Result<BulkResultDto, AppError> {
    let config = state.config();
    let git = GitBinary::resolve()?;
    let batch = git.run_batch_locked(&config.workspace.repositories, &BatchOp::Fetch)?;
    Ok(batch_to_dto(batch))
}

#[tauri::command]
pub fn pull_all(state: State<'_, AppState>) -> Result<BulkResultDto, AppError> {
    let config = state.config();
    let git = GitBinary::resolve()?;
    let batch = git.run_batch_locked(&config.workspace.repositories, &BatchOp::Pull)?;
    Ok(batch_to_dto(batch))
}
