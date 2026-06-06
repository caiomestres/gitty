use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::repository::Workspace;

/// A named, ordered sequence of Steps targeting a selection of Repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDef {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    #[serde(default = "default_backoff")]
    pub backoff_seconds: u64,
}

fn default_backoff() -> u64 {
    2
}

/// A single unit of work inside a Macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub kind: StepKind,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub rollback: Option<Box<Step>>,
    #[serde(default)]
    pub confirm: bool,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// Either a typed Git Operation or a Shell Command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepKind {
    GitOp(GitOp),
    Shell(ShellStep),
}

/// A first-class Git command with structured parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GitOp {
    Fetch,
    Pull,
    Checkout { branch: String },
}

/// An arbitrary shell command string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStep {
    pub command: String,
    #[serde(default)]
    pub label: Option<String>,
}

impl Workspace {
    /// Register a new Macro and return its id.
    pub fn define_macro(&mut self, name: &str, steps: Vec<Step>) -> Result<Uuid> {
        if self.macros.iter().any(|m| m.name == name) {
            return Err(CoreError::DuplicateMacroName(name.to_string()));
        }
        let id = Uuid::new_v4();
        self.macros.push(MacroDef {
            id,
            name: name.to_string(),
            steps,
            variables: HashMap::new(),
        });
        Ok(id)
    }

    /// Remove a Macro by id.
    pub fn delete_macro(&mut self, id: Uuid) -> Result<()> {
        let before = self.macros.len();
        self.macros.retain(|m| m.id != id);
        if self.macros.len() == before {
            return Err(CoreError::MacroNotFound(id.to_string()));
        }
        Ok(())
    }

    /// All Macros in the Workspace.
    pub fn list_macros(&self) -> &[MacroDef] {
        &self.macros
    }

    /// Look up a Macro by id.
    pub fn find_macro_by_id(&self, id: Uuid) -> Option<&MacroDef> {
        self.macros.iter().find(|m| m.id == id)
    }

    /// Look up a Macro by exact name or UUID string.
    pub fn find_macro(&self, name_or_id: &str) -> Option<&MacroDef> {
        if let Ok(id) = Uuid::parse_str(name_or_id) {
            if let Some(m) = self.macros.iter().find(|m| m.id == id) {
                return Some(m);
            }
        }
        self.macros.iter().find(|m| m.name == name_or_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fetch_step() -> Step {
        Step {
            kind: StepKind::GitOp(GitOp::Fetch),
            condition: None,
            rollback: None,
            confirm: false,
            retry: None,
        }
    }

    #[test]
    fn step_serde_roundtrip_without_retry() {
        let step = fetch_step();
        let json = serde_json::to_string(&step).unwrap();
        let back: Step = serde_json::from_str(&json).unwrap();
        assert!(back.retry.is_none());
        assert!(matches!(back.kind, StepKind::GitOp(GitOp::Fetch)));
    }

    #[test]
    fn step_serde_roundtrip_with_retry() {
        let step = Step {
            kind: StepKind::GitOp(GitOp::Pull),
            condition: None,
            rollback: None,
            confirm: false,
            retry: Some(RetryConfig {
                max_attempts: 3,
                backoff_seconds: 2,
            }),
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(back.retry.as_ref().unwrap().max_attempts, 3);
        assert_eq!(back.retry.as_ref().unwrap().backoff_seconds, 2);
    }

    #[test]
    fn step_serde_backward_compat_omitted_retry() {
        let json = r#"{"type":"git_op","op":"fetch"}"#;
        let step: Step = serde_json::from_str(json).unwrap();
        assert!(step.retry.is_none());
    }

    #[test]
    fn define_and_list_macros() {
        let mut ws = Workspace::default();
        let id = ws.define_macro("sync", vec![fetch_step()]).unwrap();
        assert_eq!(ws.list_macros().len(), 1);
        assert_eq!(ws.list_macros()[0].id, id);
    }

    #[test]
    fn define_macro_rejects_duplicate_names() {
        let mut ws = Workspace::default();
        ws.define_macro("sync", vec![fetch_step()]).unwrap();
        let err = ws.define_macro("sync", vec![fetch_step()]).unwrap_err();
        assert!(matches!(err, CoreError::DuplicateMacroName(_)));
    }

    #[test]
    fn find_macro_by_name_or_id() {
        let mut ws = Workspace::default();
        let id = ws.define_macro("sync", vec![fetch_step()]).unwrap();
        assert!(ws.find_macro("sync").is_some());
        assert!(ws.find_macro(&id.to_string()).is_some());
    }

    #[test]
    fn delete_macro() {
        let mut ws = Workspace::default();
        let id = ws.define_macro("sync", vec![fetch_step()]).unwrap();
        ws.delete_macro(id).unwrap();
        assert!(ws.list_macros().is_empty());
    }
}
