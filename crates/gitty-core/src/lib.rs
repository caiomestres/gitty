//! Core domain logic for Gitty — no Tauri or CLI-framework dependencies.
//!
//! Read operations use `git2`; write operations (added in a later slice) shell
//! out to the `git` CLI (ADR-0001). Both the CLI and the desktop app build on
//! this crate.

pub mod config;
pub mod error;
pub mod execution;
pub mod git;
pub mod group;
pub mod job;
pub mod lock;
pub mod macro_def;
pub mod reconcile;
pub mod repository;
pub mod scan;
pub mod selection;
pub mod tag;

pub use config::Config;
pub use error::{CoreError, Result};
pub use execution::execute_macro;
pub use group::{Group, GroupTreeNode, UNGROUPED_GROUP_NAME};
pub use job::{Job, JobStatus, StepResult};
pub use macro_def::{GitOp, MacroDef, ShellStep, Step, StepKind};
pub use reconcile::ReconcileReport;
pub use selection::Selection;
pub use tag::FAVORITE_TAG;

use std::path::Path;

/// Add `root` as a Scan Root (if new), scan it, and reconcile the results into
/// the workspace registry. The caller is responsible for persisting the Config.
pub fn scan_and_reconcile(config: &mut Config, root: &Path) -> Result<ReconcileReport> {
    let canonical =
        dunce::canonicalize(root).map_err(|_| CoreError::PathNotFound(root.to_path_buf()))?;
    config.workspace.add_scan_root(canonical.clone());
    let discovered = scan::scan(&canonical)?;
    Ok(reconcile::reconcile(&mut config.workspace, discovered))
}
