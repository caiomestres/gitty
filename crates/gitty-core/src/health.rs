use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::git::read::{self, RepositoryStatus};
use crate::repository::{Repository, RepositoryState};

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_id: String,
    pub severity: CheckSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealth {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub checks: Vec<CheckResult>,
    pub worst_severity: CheckSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceHealth {
    pub score: f64,
    pub total_repos: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub healthy_count: usize,
    pub repositories: Vec<RepositoryHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub stale_days_warning: u32,
    pub stale_days_critical: u32,
    pub diverged_warning: usize,
    pub diverged_critical: usize,
}

impl Default for HealthThresholds {
    /// Defaults: stale warning at 7 days, critical at 14 days;
    /// diverged warning at 5 commits behind, critical at 20.
    fn default() -> Self {
        Self {
            stale_days_warning: 7,
            stale_days_critical: 14,
            diverged_warning: 5,
            diverged_critical: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Health Check Functions
// ---------------------------------------------------------------------------

fn check_stale(
    status: &RepositoryStatus,
    thresholds: &HealthThresholds,
    now: OffsetDateTime,
) -> CheckResult {
    let head = match &status.head {
        Some(h) => h,
        None => {
            return CheckResult {
                check_id: "stale".into(),
                severity: CheckSeverity::Healthy,
                message: "No commits — stale check skipped".into(),
            };
        }
    };

    let commit_date = match OffsetDateTime::parse(&head.date, &Rfc3339) {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                check_id: "stale".into(),
                severity: CheckSeverity::Warning,
                message: "Could not parse HEAD commit date".into(),
            };
        }
    };

    let age_days = (now - commit_date).whole_days().unsigned_abs() as u32;

    if age_days >= thresholds.stale_days_critical {
        CheckResult {
            check_id: "stale".into(),
            severity: CheckSeverity::Critical,
            message: format!(
                "HEAD is {age_days} days old (critical threshold: {} days)",
                thresholds.stale_days_critical
            ),
        }
    } else if age_days >= thresholds.stale_days_warning {
        CheckResult {
            check_id: "stale".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "HEAD is {age_days} days old (warning threshold: {} days)",
                thresholds.stale_days_warning
            ),
        }
    } else {
        CheckResult {
            check_id: "stale".into(),
            severity: CheckSeverity::Healthy,
            message: format!("HEAD is {age_days} days old"),
        }
    }
}

fn check_diverged(status: &RepositoryStatus, thresholds: &HealthThresholds) -> CheckResult {
    let upstream = match &status.upstream {
        Some(u) => u,
        None => {
            return CheckResult {
                check_id: "diverged".into(),
                severity: CheckSeverity::Healthy,
                message: "No upstream configured — diverged check skipped".into(),
            };
        }
    };

    let behind = upstream.behind;

    if behind >= thresholds.diverged_critical {
        CheckResult {
            check_id: "diverged".into(),
            severity: CheckSeverity::Critical,
            message: format!(
                "{behind} commits behind upstream (critical threshold: {})",
                thresholds.diverged_critical
            ),
        }
    } else if behind >= thresholds.diverged_warning {
        CheckResult {
            check_id: "diverged".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "{behind} commits behind upstream (warning threshold: {})",
                thresholds.diverged_warning
            ),
        }
    } else {
        CheckResult {
            check_id: "diverged".into(),
            severity: CheckSeverity::Healthy,
            message: format!("{behind} commits behind upstream"),
        }
    }
}

fn check_dirty(status: &RepositoryStatus) -> CheckResult {
    if status.dirty {
        CheckResult {
            check_id: "dirty".into(),
            severity: CheckSeverity::Warning,
            message: "Working tree has uncommitted changes".into(),
        }
    } else {
        CheckResult {
            check_id: "dirty".into(),
            severity: CheckSeverity::Healthy,
            message: "Working tree is clean".into(),
        }
    }
}

fn check_detached(status: &RepositoryStatus) -> CheckResult {
    if status.detached {
        CheckResult {
            check_id: "detached".into(),
            severity: CheckSeverity::Critical,
            message: "HEAD is detached".into(),
        }
    } else {
        CheckResult {
            check_id: "detached".into(),
            severity: CheckSeverity::Healthy,
            message: "HEAD is on a branch".into(),
        }
    }
}

/// Run all built-in health checks against a single repository status.
pub fn evaluate_checks(
    status: &RepositoryStatus,
    thresholds: &HealthThresholds,
    now: OffsetDateTime,
) -> Vec<CheckResult> {
    vec![
        check_stale(status, thresholds, now),
        check_diverged(status, thresholds),
        check_dirty(status),
        check_detached(status),
    ]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect `RepositoryStatus` for each repository, silently skipping
/// repos whose status cannot be read.
pub fn collect_statuses(repos: &[&Repository]) -> HashMap<Uuid, RepositoryStatus> {
    let mut statuses = HashMap::new();
    for repo in repos {
        if let Ok(s) = read::read_status(&repo.path) {
            statuses.insert(repo.id, s);
        }
    }
    statuses
}

/// Filter a repository list to only Active repos.
pub fn active_repos(repos: &[Repository]) -> Vec<&Repository> {
    repos
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect()
}

fn worst(checks: &[CheckResult]) -> CheckSeverity {
    if checks.iter().any(|c| c.severity == CheckSeverity::Critical) {
        CheckSeverity::Critical
    } else if checks.iter().any(|c| c.severity == CheckSeverity::Warning) {
        CheckSeverity::Warning
    } else {
        CheckSeverity::Healthy
    }
}

// ---------------------------------------------------------------------------
// Evaluation Functions
// ---------------------------------------------------------------------------

/// Evaluate a single Repository against the built-in checks.
pub fn evaluate_repository(
    repo: &Repository,
    status: &RepositoryStatus,
    thresholds: &HealthThresholds,
    now: OffsetDateTime,
) -> RepositoryHealth {
    let results = evaluate_checks(status, thresholds, now);
    let worst_severity = worst(&results);

    RepositoryHealth {
        repo_id: repo.id,
        repo_name: repo.display_name().to_string(),
        checks: results,
        worst_severity,
    }
}

/// Evaluate a set of repositories. Repos whose status is absent from
/// `statuses` are silently skipped.
/// Score = (repos not critical / total evaluated) * 100.
/// Empty input → score is -1.0 (displayed as N/A).
pub fn evaluate_workspace(
    repos: &[&Repository],
    statuses: &HashMap<Uuid, RepositoryStatus>,
    thresholds: &HealthThresholds,
) -> WorkspaceHealth {
    if repos.is_empty() {
        return WorkspaceHealth {
            score: -1.0,
            total_repos: 0,
            critical_count: 0,
            warning_count: 0,
            healthy_count: 0,
            repositories: Vec::new(),
        };
    }

    let now = OffsetDateTime::now_utc();
    let mut repo_healths = Vec::new();
    for repo in repos {
        if let Some(status) = statuses.get(&repo.id) {
            repo_healths.push(evaluate_repository(repo, status, thresholds, now));
        }
    }

    let critical_count = repo_healths
        .iter()
        .filter(|rh| rh.worst_severity == CheckSeverity::Critical)
        .count();
    let warning_count = repo_healths
        .iter()
        .filter(|rh| rh.worst_severity == CheckSeverity::Warning)
        .count();
    let healthy_count = repo_healths
        .iter()
        .filter(|rh| rh.worst_severity == CheckSeverity::Healthy)
        .count();
    let total = repo_healths.len();
    let not_critical = total - critical_count;
    let score = if total > 0 {
        (not_critical as f64 / total as f64) * 100.0
    } else {
        -1.0
    };

    WorkspaceHealth {
        score,
        total_repos: total,
        critical_count,
        warning_count,
        healthy_count,
        repositories: repo_healths,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::read::{CommitSummary, Upstream};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    fn make_repo(state: RepositoryState) -> Repository {
        let mut r = Repository::new(PathBuf::from("/test/repo"), Some("fp".into()));
        r.state = state;
        r
    }

    fn fresh_status() -> RepositoryStatus {
        RepositoryStatus {
            branch: Some("main".into()),
            detached: false,
            dirty: false,
            upstream: Some(Upstream {
                name: "origin/main".into(),
                ahead: 0,
                behind: 0,
            }),
            head: Some(CommitSummary {
                short_id: "abc1234".into(),
                author: "Test".into(),
                date: OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
                subject: "latest commit".into(),
            }),
            changed_files: vec![],
        }
    }

    fn stale_status(days_ago: i64) -> RepositoryStatus {
        let past = OffsetDateTime::now_utc() - time::Duration::days(days_ago);
        let mut s = fresh_status();
        if let Some(h) = s.head.as_mut() {
            h.date = past.format(&Rfc3339).unwrap();
        }
        s
    }

    fn diverged_status(behind: usize) -> RepositoryStatus {
        let mut s = fresh_status();
        if let Some(u) = s.upstream.as_mut() {
            u.behind = behind;
        }
        s
    }

    fn dirty_status() -> RepositoryStatus {
        let mut s = fresh_status();
        s.dirty = true;
        s
    }

    fn detached_status() -> RepositoryStatus {
        let mut s = fresh_status();
        s.detached = true;
        s.branch = None;
        s
    }

    fn no_upstream_status() -> RepositoryStatus {
        let mut s = fresh_status();
        s.upstream = None;
        s
    }

    fn no_commits_status() -> RepositoryStatus {
        RepositoryStatus {
            branch: Some("main".into()),
            detached: false,
            dirty: false,
            upstream: None,
            head: None,
            changed_files: vec![],
        }
    }

    #[test]
    fn stale_check_healthy_when_recent() {
        let thresholds = HealthThresholds::default();
        let result = check_stale(&fresh_status(), &thresholds, now());
        assert_eq!(result.severity, CheckSeverity::Healthy);
        assert_eq!(result.check_id, "stale");
    }

    #[test]
    fn stale_check_warning_at_threshold() {
        let thresholds = HealthThresholds::default();
        let result = check_stale(&stale_status(8), &thresholds, now());
        assert_eq!(result.severity, CheckSeverity::Warning);
    }

    #[test]
    fn stale_check_critical_at_double_threshold() {
        let thresholds = HealthThresholds::default();
        let result = check_stale(&stale_status(15), &thresholds, now());
        assert_eq!(result.severity, CheckSeverity::Critical);
    }

    #[test]
    fn stale_check_skips_empty_repo() {
        let thresholds = HealthThresholds::default();
        let result = check_stale(&no_commits_status(), &thresholds, now());
        assert_eq!(result.severity, CheckSeverity::Healthy);
    }

    #[test]
    fn diverged_check_healthy_when_in_sync() {
        let thresholds = HealthThresholds::default();
        let result = check_diverged(&fresh_status(), &thresholds);
        assert_eq!(result.severity, CheckSeverity::Healthy);
    }

    #[test]
    fn diverged_check_warning() {
        let thresholds = HealthThresholds::default();
        let result = check_diverged(&diverged_status(6), &thresholds);
        assert_eq!(result.severity, CheckSeverity::Warning);
    }

    #[test]
    fn diverged_check_critical() {
        let thresholds = HealthThresholds::default();
        let result = check_diverged(&diverged_status(25), &thresholds);
        assert_eq!(result.severity, CheckSeverity::Critical);
    }

    #[test]
    fn diverged_check_skips_no_upstream() {
        let thresholds = HealthThresholds::default();
        let result = check_diverged(&no_upstream_status(), &thresholds);
        assert_eq!(result.severity, CheckSeverity::Healthy);
        assert!(result.message.contains("skipped"));
    }

    #[test]
    fn dirty_check_warning() {
        let result = check_dirty(&dirty_status());
        assert_eq!(result.severity, CheckSeverity::Warning);
    }

    #[test]
    fn dirty_check_healthy() {
        let result = check_dirty(&fresh_status());
        assert_eq!(result.severity, CheckSeverity::Healthy);
    }

    #[test]
    fn detached_check_critical() {
        let result = check_detached(&detached_status());
        assert_eq!(result.severity, CheckSeverity::Critical);
    }

    #[test]
    fn detached_check_healthy() {
        let result = check_detached(&fresh_status());
        assert_eq!(result.severity, CheckSeverity::Healthy);
    }

    #[test]
    fn evaluate_repository_computes_worst_severity() {
        let repo = make_repo(RepositoryState::Active);
        let thresholds = HealthThresholds::default();
        let health = evaluate_repository(&repo, &detached_status(), &thresholds, now());
        assert_eq!(health.worst_severity, CheckSeverity::Critical);
    }

    #[test]
    fn evaluate_workspace_score_with_critical() {
        let repos: Vec<Repository> = (0..10)
            .map(|i| {
                let mut r =
                    Repository::new(PathBuf::from(format!("/test/repo{i}")), Some("fp".into()));
                r.state = RepositoryState::Active;
                r
            })
            .collect();

        let thresholds = HealthThresholds::default();
        let repo_refs: Vec<&Repository> = repos.iter().collect();

        let mut statuses: HashMap<Uuid, RepositoryStatus> =
            repos.iter().map(|r| (r.id, fresh_status())).collect();

        for i in 0..3 {
            statuses.insert(repos[i].id, detached_status());
        }

        let health = evaluate_workspace(&repo_refs, &statuses, &thresholds);
        assert_eq!(health.total_repos, 10);
        assert_eq!(health.critical_count, 3);
        assert!((health.score - 70.0).abs() < 0.01);
    }

    #[test]
    fn evaluate_workspace_all_healthy() {
        let repos: Vec<Repository> = (0..5)
            .map(|i| {
                let mut r =
                    Repository::new(PathBuf::from(format!("/test/repo{i}")), Some("fp".into()));
                r.state = RepositoryState::Active;
                r
            })
            .collect();

        let thresholds = HealthThresholds::default();
        let repo_refs: Vec<&Repository> = repos.iter().collect();
        let statuses: HashMap<Uuid, RepositoryStatus> =
            repos.iter().map(|r| (r.id, fresh_status())).collect();

        let health = evaluate_workspace(&repo_refs, &statuses, &thresholds);
        assert!((health.score - 100.0).abs() < 0.01);
        assert_eq!(health.healthy_count, 5);
    }

    #[test]
    fn evaluate_workspace_skips_missing_repos() {
        let mut repos = vec![make_repo(RepositoryState::Active)];
        let mut missing = make_repo(RepositoryState::Missing);
        missing.id = Uuid::new_v4();
        repos.push(missing);

        let thresholds = HealthThresholds::default();
        let repo_refs: Vec<&Repository> = repos
            .iter()
            .filter(|r| r.state == RepositoryState::Active)
            .collect();
        let statuses: HashMap<Uuid, RepositoryStatus> =
            [(repos[0].id, fresh_status())].into_iter().collect();

        let health = evaluate_workspace(&repo_refs, &statuses, &thresholds);
        assert_eq!(health.total_repos, 1);
    }

    #[test]
    fn evaluate_workspace_zero_repos_returns_na() {
        let repo_refs: Vec<&Repository> = vec![];
        let thresholds = HealthThresholds::default();
        let statuses: HashMap<Uuid, RepositoryStatus> = HashMap::new();

        let health = evaluate_workspace(&repo_refs, &statuses, &thresholds);
        assert!(health.score < 0.0);
        assert_eq!(health.total_repos, 0);
    }

    #[test]
    fn health_thresholds_default_values() {
        let t = HealthThresholds::default();
        assert_eq!(t.stale_days_warning, 7);
        assert_eq!(t.stale_days_critical, 14);
        assert_eq!(t.diverged_warning, 5);
        assert_eq!(t.diverged_critical, 20);
    }

    #[test]
    fn check_severity_serializes_lowercase() {
        let json = serde_json::to_string(&CheckSeverity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
    }
}
