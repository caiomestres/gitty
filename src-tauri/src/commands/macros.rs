use std::collections::HashMap;

use gitty_core::git::write::GitBinary;
use gitty_core::job::JobStatus;
use gitty_core::macro_def::{GitOp, ShellStep, Step, StepKind};
use gitty_core::selection::Selection;
use gitty_core::{execute_macro, CoreError};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use super::parse_uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDto {
    id: String,
    name: String,
    steps: Vec<StepDto>,
    variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDto {
    kind: StepKindDto,
    condition: Option<String>,
    rollback: Option<Box<StepDto>>,
    confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepKindDto {
    #[serde(rename = "git_op")]
    GitOp { op: String, branch: Option<String> },
    #[serde(rename = "shell")]
    Shell {
        command: String,
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum SelectionDto {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "group")]
    Group { id: String },
    #[serde(rename = "tag")]
    Tag { name: String },
    #[serde(rename = "multiple")]
    Multiple { ids: Vec<String> },
}

#[derive(Debug, Clone, Serialize)]
pub struct JobDto {
    id: String,
    repo_id: String,
    repo_name: String,
    status: String,
    error: Option<String>,
    step_results: Vec<StepResultDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResultDto {
    step_index: usize,
    status: String,
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn step_to_dto(step: &Step) -> StepDto {
    let kind = match &step.kind {
        StepKind::GitOp(op) => {
            let (op_name, branch) = match op {
                GitOp::Fetch => ("fetch".to_string(), None),
                GitOp::Pull => ("pull".to_string(), None),
                GitOp::Checkout { branch } => ("checkout".to_string(), Some(branch.clone())),
            };
            StepKindDto::GitOp {
                op: op_name,
                branch,
            }
        }
        StepKind::Shell(shell) => StepKindDto::Shell {
            command: shell.command.clone(),
            label: shell.label.clone(),
        },
    };

    StepDto {
        kind,
        condition: step.condition.clone(),
        rollback: step.rollback.as_ref().map(|r| Box::new(step_to_dto(r))),
        confirm: step.confirm,
    }
}

fn step_dto_to_core(dto: StepDto) -> Result<Step, AppError> {
    let kind = match dto.kind {
        StepKindDto::GitOp { op, branch } => {
            let git_op = match op.as_str() {
                "fetch" => GitOp::Fetch,
                "pull" => GitOp::Pull,
                "checkout" => GitOp::Checkout {
                    branch: branch.ok_or_else(|| AppError {
                        code: "invalid_step".into(),
                        message: "checkout step requires a branch".into(),
                    })?,
                },
                other => {
                    return Err(AppError {
                        code: "invalid_step".into(),
                        message: format!("unknown git operation: {other}"),
                    })
                }
            };
            StepKind::GitOp(git_op)
        }
        StepKindDto::Shell { command, label } => StepKind::Shell(ShellStep { command, label }),
    };

    let rollback = dto
        .rollback
        .map(|r| step_dto_to_core(*r).map(Box::new))
        .transpose()?;

    Ok(Step {
        kind,
        condition: dto.condition,
        rollback,
        confirm: dto.confirm,
    })
}

fn macro_to_dto(m: &gitty_core::MacroDef) -> MacroDto {
    MacroDto {
        id: m.id.to_string(),
        name: m.name.clone(),
        steps: m.steps.iter().map(step_to_dto).collect(),
        variables: m.variables.clone(),
    }
}

fn job_status_str(status: &JobStatus) -> (String, Option<String>) {
    match status {
        JobStatus::Pending => ("pending".into(), None),
        JobStatus::Running => ("running".into(), None),
        JobStatus::Success => ("success".into(), None),
        JobStatus::Failed { error } => ("failed".into(), Some(error.clone())),
        JobStatus::Skipped { reason } => ("skipped".into(), Some(reason.clone())),
        JobStatus::Cancelled => ("cancelled".into(), None),
    }
}

fn selection_to_core(dto: SelectionDto) -> Result<Selection, AppError> {
    match dto {
        SelectionDto::All => Ok(Selection::All),
        SelectionDto::Group { id } => Ok(Selection::Group(parse_uuid(&id)?)),
        SelectionDto::Tag { name } => Ok(Selection::Tag(name)),
        SelectionDto::Multiple { ids } => {
            let uuids = ids
                .iter()
                .map(|id| parse_uuid(id))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Selection::Multiple(uuids))
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_macros(state: State<'_, AppState>) -> Result<Vec<MacroDto>, AppError> {
    let config = state.config();
    Ok(config
        .workspace
        .list_macros()
        .iter()
        .map(macro_to_dto)
        .collect())
}

#[tauri::command]
pub fn get_macro(state: State<'_, AppState>, name_or_id: String) -> Result<MacroDto, AppError> {
    let config = state.config();
    let m = config
        .workspace
        .find_macro(&name_or_id)
        .ok_or_else(|| AppError::from(CoreError::MacroNotFound(name_or_id)))?;
    Ok(macro_to_dto(m))
}

#[tauri::command]
pub fn define_macro(
    state: State<'_, AppState>,
    name: String,
    steps: Vec<StepDto>,
    variables: HashMap<String, String>,
) -> Result<MacroDto, AppError> {
    let core_steps = steps
        .into_iter()
        .map(step_dto_to_core)
        .collect::<Result<Vec<_>, _>>()?;

    state.with_config_write(|config| {
        let id = config.workspace.define_macro(&name, core_steps)?;
        if let Some(m) = config.workspace.macros.iter_mut().find(|m| m.id == id) {
            m.variables = variables;
        }
        let m = config.workspace.find_macro(&id.to_string()).unwrap();
        Ok(macro_to_dto(m))
    })
}

#[tauri::command]
pub fn delete_macro(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.with_config_write(|config| {
        let uuid = parse_uuid(&id)?;
        config.workspace.delete_macro(uuid)?;
        Ok(())
    })
}

#[tauri::command]
pub fn run_macro(
    state: State<'_, AppState>,
    name_or_id: String,
    selection: SelectionDto,
) -> Result<Vec<JobDto>, AppError> {
    let config = state.config();
    let macro_def = config
        .workspace
        .find_macro(&name_or_id)
        .ok_or_else(|| AppError::from(CoreError::MacroNotFound(name_or_id)))?
        .clone();

    let sel = selection_to_core(selection)?;
    let repos = sel.resolve(&config.workspace);
    let git = GitBinary::resolve()?;
    let jobs = execute_macro(&macro_def, &repos, &git);

    Ok(jobs
        .iter()
        .map(|job| {
            let (status, error) = job_status_str(&job.status);
            let rn = config
                .workspace
                .find_by_id(job.repo_id)
                .map(|r| r.display_name().to_string())
                .unwrap_or_else(|| job.repo_id.to_string());

            JobDto {
                id: job.id.to_string(),
                repo_id: job.repo_id.to_string(),
                repo_name: rn,
                status,
                error,
                step_results: job
                    .step_results
                    .iter()
                    .map(|sr| {
                        let (s, _) = job_status_str(&sr.status);
                        StepResultDto {
                            step_index: sr.step_index,
                            status: s,
                            output: sr.output.clone(),
                        }
                    })
                    .collect(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitty_core::macro_def::{GitOp, Step, StepKind};

    #[test]
    fn step_roundtrip_git_op() {
        let step = Step {
            kind: StepKind::GitOp(GitOp::Fetch),
            condition: None,
            rollback: None,
            confirm: false,
        };
        let dto = step_to_dto(&step);
        let back = step_dto_to_core(dto).unwrap();
        assert!(matches!(back.kind, StepKind::GitOp(GitOp::Fetch)));
    }

    #[test]
    fn step_roundtrip_shell() {
        let step = Step {
            kind: StepKind::Shell(ShellStep {
                command: "echo hi".into(),
                label: Some("greet".into()),
            }),
            condition: Some("true".into()),
            rollback: None,
            confirm: true,
        };
        let dto = step_to_dto(&step);
        let back = step_dto_to_core(dto).unwrap();
        assert!(matches!(back.kind, StepKind::Shell(_)));
        assert_eq!(back.condition.as_deref(), Some("true"));
        assert!(back.confirm);
    }

    #[test]
    fn step_dto_checkout_requires_branch() {
        let dto = StepDto {
            kind: StepKindDto::GitOp {
                op: "checkout".into(),
                branch: None,
            },
            condition: None,
            rollback: None,
            confirm: false,
        };
        assert!(step_dto_to_core(dto).is_err());
    }

    #[test]
    fn macro_dto_includes_variables() {
        let m = gitty_core::MacroDef {
            id: uuid::Uuid::new_v4(),
            name: "test".into(),
            steps: vec![],
            variables: HashMap::from([("key".into(), "val".into())]),
        };
        let dto = macro_to_dto(&m);
        assert_eq!(dto.variables.get("key").unwrap(), "val");
    }
}
