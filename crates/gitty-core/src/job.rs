use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state of a Job (one Macro run on one Repository).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed { error: String },
    Skipped { reason: String },
    Cancelled,
}

/// Outcome of a single Step within a Job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub status: JobStatus,
    #[serde(default)]
    pub output: Option<String>,
}

/// One Macro execution on one Repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub macro_id: Uuid,
    pub repo_id: Uuid,
    pub status: JobStatus,
    #[serde(default)]
    pub step_results: Vec<StepResult>,
}

impl Job {
    pub fn new(macro_id: Uuid, repo_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            macro_id,
            repo_id,
            status: JobStatus::Pending,
            step_results: Vec::new(),
        }
    }
}
