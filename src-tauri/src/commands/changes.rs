use std::collections::HashSet;

use gitty_core::changes::{self, Grouping, TimeWindow};
use gitty_core::repository::RepositoryState;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntryDto {
    pub commit_hash: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub branch: String,
    pub repo_id: String,
    pub repo_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedChangesDto {
    pub groups: Vec<ChangeGroupDto>,
    pub total_commits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeGroupDto {
    pub key: String,
    pub entries: Vec<ChangeEntryDto>,
}

fn parse_window(s: &str) -> TimeWindow {
    match s {
        "day" => TimeWindow::Day,
        "month" => TimeWindow::Month,
        _ => TimeWindow::Week,
    }
}

fn parse_grouping(s: &str) -> Grouping {
    match s {
        "author" => Grouping::Author,
        "branch" => Grouping::Branch,
        _ => Grouping::Repository,
    }
}

#[tauri::command]
pub fn get_changes(
    state: State<'_, AppState>,
    window: String,
    grouping: String,
    all_branches_repos: Vec<String>,
) -> Result<GroupedChangesDto, AppError> {
    let config = state.config();
    let repos: Vec<_> = config
        .workspace
        .repositories
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect();

    let all_branches: HashSet<uuid::Uuid> = all_branches_repos
        .iter()
        .filter_map(|id| uuid::Uuid::parse_str(id).ok())
        .collect();

    let tw = parse_window(&window);
    let grp = parse_grouping(&grouping);

    let entries = changes::scan_changes(&repos, tw, &all_branches)?;
    let total_commits = entries.len();
    let grouped = changes::group_changes(&entries, grp);

    let groups = grouped
        .into_iter()
        .map(|(key, refs)| ChangeGroupDto {
            key,
            entries: refs
                .into_iter()
                .map(|e| ChangeEntryDto {
                    commit_hash: e.commit_hash.clone(),
                    author: e.author.clone(),
                    date: e.date.clone(),
                    subject: e.subject.clone(),
                    branch: e.branch.clone(),
                    repo_id: e.repo_id.to_string(),
                    repo_name: e.repo_name.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(GroupedChangesDto {
        groups,
        total_commits,
    })
}
