use std::collections::HashMap;

use anyhow::{Context, Result};
use gitty_core::config::Config;
use gitty_core::git::read;
use gitty_core::git::write::match_repo;
use gitty_core::health::{self, CheckSeverity};
use gitty_core::repository::RepositoryState;
use time::OffsetDateTime;

pub fn cmd_health(target: Option<&str>) -> Result<()> {
    let config = Config::load().context("loading config")?;
    let repos = &config.workspace.repositories;
    if repos.is_empty() {
        println!("No repositories tracked yet. Run `gitty scan <path>` to discover some.");
        return Ok(());
    }

    let config_dir = gitty_core::config::paths::config_dir().context("resolving config dir")?;
    let thresholds = &config.workspace.health_thresholds;
    let checks = health::default_checks();

    if let Some(target) = target {
        let repo = match_repo(repos, target).map_err(|e| anyhow::anyhow!("{e}"))?;
        if repo.state == RepositoryState::Missing {
            println!("{} [missing]", repo.path.display());
            return Ok(());
        }
        let status = read::read_status(&repo.path)
            .with_context(|| format!("reading status of {}", repo.path.display()))?;
        let now = OffsetDateTime::now_utc();
        let health = health::evaluate_repository(repo, &status, &checks, thresholds, now);
        let name = repo.display_name();
        println!("Repository: {name}");
        println!(
            "  Worst severity: {}",
            severity_label(health.worst_severity)
        );
        for check in &health.checks {
            println!(
                "  [{}] {} — {}",
                severity_dot(check.severity),
                check.check_id,
                check.message
            );
        }
        return Ok(());
    }

    let cached = gitty_core::health_cache::load(&config_dir);
    if let Some(cached) = &cached {
        println!("(cached — last evaluated: {})\n", cached.last_evaluated);
        print_workspace_health(&cached.workspace_health);
        return Ok(());
    }

    let active_repos: Vec<_> = repos
        .iter()
        .filter(|r| r.state == RepositoryState::Active)
        .collect();
    let mut statuses = HashMap::new();
    for repo in &active_repos {
        match read::read_status(&repo.path) {
            Ok(s) => {
                statuses.insert(repo.id, s);
            }
            Err(e) => eprintln!("warning: could not read {}: {e}", repo.path.display()),
        }
    }

    let workspace_health = health::evaluate_workspace(repos, &statuses, &checks, thresholds);
    let _ = gitty_core::health_cache::save(&workspace_health, &config_dir);
    print_workspace_health(&workspace_health);
    Ok(())
}

fn print_workspace_health(wh: &gitty_core::WorkspaceHealth) {
    if wh.score < 0.0 {
        println!("Workspace Health: N/A (no active repositories)");
        return;
    }
    println!("Workspace Health: {:.0}%", wh.score);
    println!(
        "  {} total | {} healthy | {} warning | {} critical\n",
        wh.total_repos, wh.healthy_count, wh.warning_count, wh.critical_count
    );

    for rh in &wh.repositories {
        let sev = severity_dot(rh.worst_severity);
        println!(
            "{sev} {:<24} {}",
            rh.repo_name,
            severity_label(rh.worst_severity)
        );
    }
}

fn severity_dot(s: CheckSeverity) -> &'static str {
    match s {
        CheckSeverity::Healthy => "\u{25CF}",
        CheckSeverity::Warning => "\u{25CB}",
        CheckSeverity::Critical => "\u{25C6}",
    }
}

fn severity_label(s: CheckSeverity) -> &'static str {
    match s {
        CheckSeverity::Healthy => "healthy",
        CheckSeverity::Warning => "warning",
        CheckSeverity::Critical => "critical",
    }
}
