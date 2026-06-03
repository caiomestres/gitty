use std::process::Command;

use crate::git::write::{GitBinary, GitResult};
use crate::job::{Job, JobStatus, StepResult};
use crate::macro_def::{GitOp, MacroDef, ShellStep, Step, StepKind};
use crate::repository::{Repository, RepositoryState};

/// Execute `macro_def` against `repos` sequentially (one Job per Repository).
pub fn execute_macro(macro_def: &MacroDef, repos: &[&Repository], git: &GitBinary) -> Vec<Job> {
    repos
        .iter()
        .map(|repo| execute_macro_on_repo(macro_def, repo, git))
        .collect()
}

fn execute_macro_on_repo(macro_def: &MacroDef, repo: &Repository, git: &GitBinary) -> Job {
    let mut job = Job::new(macro_def.id, repo.id);
    job.status = JobStatus::Running;

    if repo.state == RepositoryState::Missing {
        job.status = JobStatus::Skipped {
            reason: "repository path not found".into(),
        };
        return job;
    }

    for (index, step) in macro_def.steps.iter().enumerate() {
        if !should_run_step(step) {
            job.step_results.push(StepResult {
                step_index: index,
                status: JobStatus::Skipped {
                    reason: "condition not met".into(),
                },
                output: None,
            });
            continue;
        }

        let result = execute_step(index, step, repo, git);
        let failed = matches!(result.status, JobStatus::Failed { .. });
        job.step_results.push(result);

        if failed {
            if let Some(rollback) = &step.rollback {
                let rollback_result = execute_step(index, rollback, repo, git);
                job.step_results.push(StepResult {
                    step_index: index,
                    status: rollback_result.status,
                    output: rollback_result.output.map(|o| format!("rollback: {o}")),
                });
            }
            job.status = JobStatus::Failed {
                error: format!("step {index} failed"),
            };
            return job;
        }
    }

    job.status = JobStatus::Success;
    job
}

/// A step runs unless its condition is explicitly set to a falsy literal.
/// This is intentionally a simple boolean flag — not an expression evaluator.
fn should_run_step(step: &Step) -> bool {
    !matches!(step.condition.as_deref(), Some("false" | "0"))
}

fn execute_step(step_index: usize, step: &Step, repo: &Repository, git: &GitBinary) -> StepResult {
    match &step.kind {
        StepKind::GitOp(op) => execute_git_op(step_index, op, repo, git),
        StepKind::Shell(shell) => execute_shell(step_index, shell, repo),
    }
}

fn execute_git_op(step_index: usize, op: &GitOp, repo: &Repository, git: &GitBinary) -> StepResult {
    let git_result = match op {
        GitOp::Fetch => git.fetch(&repo.path),
        GitOp::Pull => git.pull(&repo.path),
        GitOp::Checkout { branch } => git.checkout(&repo.path, branch),
    };

    match git_result {
        Ok(GitResult::Success(output)) => StepResult {
            step_index,
            status: JobStatus::Success,
            output: Some(combine_git_output(&output.stdout, &output.stderr)),
        },
        Ok(GitResult::Failed { output, category }) => StepResult {
            step_index,
            status: JobStatus::Failed {
                error: category.to_string(),
            },
            output: Some(combine_git_output(&output.stdout, &output.stderr)),
        },
        Err(e) => StepResult {
            step_index,
            status: JobStatus::Failed {
                error: e.to_string(),
            },
            output: None,
        },
    }
}

fn execute_shell(step_index: usize, shell: &ShellStep, repo: &Repository) -> StepResult {
    let output = Command::new(if cfg!(windows) { "cmd" } else { "sh" })
        .arg(if cfg!(windows) { "/C" } else { "-c" })
        .arg(&shell.command)
        .current_dir(&repo.path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let text = combine_git_output(
                &String::from_utf8_lossy(&out.stdout),
                &String::from_utf8_lossy(&out.stderr),
            );
            if out.status.success() {
                StepResult {
                    step_index,
                    status: JobStatus::Success,
                    output: Some(text),
                }
            } else {
                StepResult {
                    step_index,
                    status: JobStatus::Failed {
                        error: format!("shell command exited with {}", out.status),
                    },
                    output: Some(text),
                }
            }
        }
        Err(e) => StepResult {
            step_index,
            status: JobStatus::Failed {
                error: e.to_string(),
            },
            output: None,
        },
    }
}

fn combine_git_output(stdout: &str, stderr: &str) -> String {
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout.to_string(),
        (true, false) => stderr.to_string(),
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git::test_helpers::init_test_repo;
    use crate::macro_def::{GitOp, StepKind};
    use uuid::Uuid;

    #[test]
    fn execute_fetch_macro_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());
        let repo = Repository::new(dir.path().to_path_buf(), Some("fp".into()));
        let macro_def = MacroDef {
            id: Uuid::new_v4(),
            name: "fetch-all".into(),
            steps: vec![Step {
                kind: StepKind::GitOp(GitOp::Fetch),
                condition: None,
                rollback: None,
                confirm: false,
            }],
            variables: Default::default(),
        };
        let git = GitBinary::resolve().unwrap();
        let jobs = execute_macro(&macro_def, &[&repo], &git);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].status, JobStatus::Success);
    }

    #[test]
    fn execute_skips_missing_repo() {
        let mut repo = Repository::new("/nope".into(), None);
        repo.state = RepositoryState::Missing;
        let macro_def = MacroDef {
            id: Uuid::new_v4(),
            name: "fetch".into(),
            steps: vec![Step {
                kind: StepKind::GitOp(GitOp::Fetch),
                condition: None,
                rollback: None,
                confirm: false,
            }],
            variables: Default::default(),
        };
        let git = GitBinary::resolve().unwrap();
        let jobs = execute_macro(&macro_def, &[&repo], &git);
        assert!(matches!(jobs[0].status, JobStatus::Skipped { .. }));
    }

    #[test]
    fn execute_shell_step_runs_in_repo_dir() {
        let dir = tempfile::tempdir().unwrap();
        init_test_repo(dir.path());
        let repo = Repository::new(dir.path().to_path_buf(), None);
        let macro_def = MacroDef {
            id: Uuid::new_v4(),
            name: "echo".into(),
            steps: vec![Step {
                kind: StepKind::Shell(ShellStep {
                    command: if cfg!(windows) {
                        "echo hello".into()
                    } else {
                        "echo hello".into()
                    },
                    label: None,
                }),
                condition: None,
                rollback: None,
                confirm: false,
            }],
            variables: Default::default(),
        };
        let git = GitBinary::resolve().unwrap();
        let jobs = execute_macro(&macro_def, &[&repo], &git);
        assert_eq!(jobs[0].status, JobStatus::Success);
    }
}
