use std::path::Path;

use anyhow::{bail, Context, Result};
use gitty_core::config::Config;
use gitty_core::git::read::{self, RepositoryStatus};
use gitty_core::git::write::{match_repo, BatchOp, BatchResult, GitBinary, RepoOutcome};
use gitty_core::repository::RepositoryState;
use gitty_core::scan_and_reconcile;

pub fn cmd_scan(path: &Path) -> Result<()> {
    let mut config = Config::load().context("loading config")?;
    let report = scan_and_reconcile(&mut config, path)
        .with_context(|| format!("scanning {}", path.display()))?;
    config.save().context("saving config")?;

    println!(
        "Scanned {}: {} found, {} new, {} re-linked, {} already known, {} missing",
        path.display(),
        report.found,
        report.new,
        report.relinked,
        report.existing,
        report.missing
    );
    Ok(())
}

pub fn cmd_list() -> Result<()> {
    let config = Config::load().context("loading config")?;
    let repos = &config.workspace.repositories;
    if repos.is_empty() {
        println!("No repositories tracked yet. Run `gitty scan <path>` to discover some.");
        return Ok(());
    }
    for repo in repos {
        let state = match repo.state {
            RepositoryState::Active => "active",
            RepositoryState::Missing => "missing",
        };
        println!("{state:<8} {}", repo.path.display());
    }
    Ok(())
}

pub fn cmd_status() -> Result<()> {
    let config = Config::load().context("loading config")?;
    let repos = &config.workspace.repositories;
    if repos.is_empty() {
        println!("No repositories tracked yet. Run `gitty scan <path>` to discover some.");
        return Ok(());
    }
    for repo in repos {
        let name = repo.display_name();

        if repo.state == RepositoryState::Missing {
            println!("{name:<24} [missing] {}", repo.path.display());
            continue;
        }
        match read::read_status(&repo.path) {
            Ok(status) => println!("{}", format_status_line(name, &status)),
            Err(err) => println!("{name:<24} [error] {err}"),
        }
    }
    Ok(())
}

pub fn cmd_write(op: BatchOp<'_>, label: &str, target: Option<&str>) -> Result<()> {
    let config = Config::load().context("loading config")?;
    let repos = &config.workspace.repositories;
    if repos.is_empty() {
        println!("No repositories tracked yet. Run `gitty scan <path>` to discover some.");
        return Ok(());
    }
    let git = GitBinary::resolve().context("locating git")?;

    let batch = if let Some(target) = target {
        let repo = match_repo(repos, target);
        match repo {
            Ok(r) => git
                .run_batch_locked(std::slice::from_ref(r), &op)
                .context("acquiring locks")?,
            Err(e) => bail!("{e}"),
        }
    } else {
        git.run_batch_locked(repos, &op)
            .context("acquiring locks")?
    };

    print_batch_results(&batch, label);
    Ok(())
}

fn print_batch_results(batch: &BatchResult, label: &str) {
    for result in &batch.results {
        let name = result
            .repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<repo>");

        match &result.outcome {
            RepoOutcome::Success(_) => {
                println!("\u{2713} {name:<22} {label}ed successfully");
            }
            RepoOutcome::Failed {
                category, output, ..
            } => {
                let detail = output.stderr.lines().next().unwrap_or("").trim();
                println!("\u{2717} {name:<22} [{category}] {detail}");
            }
            RepoOutcome::Skipped { reason } => {
                println!("\u{2298} {name:<22} [skipped] {reason}");
            }
            RepoOutcome::Locked {
                holder_pid, since, ..
            } => {
                println!("\u{2298} {name:<22} [locked] held by process {holder_pid} since {since}");
            }
        }
    }

    let locked = batch.locked_count();
    if locked > 0 {
        println!(
            "\n{label}: {} ok, {} failed, {} skipped, {} locked",
            batch.success_count(),
            batch.failed_count(),
            batch.skipped_count(),
            locked,
        );
    } else {
        println!(
            "\n{label}: {} ok, {} failed, {} skipped",
            batch.success_count(),
            batch.failed_count(),
            batch.skipped_count(),
        );
    }
}

fn format_status_line(name: &str, s: &RepositoryStatus) -> String {
    let branch = if s.detached {
        "(detached)".to_string()
    } else {
        s.branch.clone().unwrap_or_else(|| "(unborn)".to_string())
    };
    let dirty = if s.dirty { "dirty" } else { "clean" };
    let tracking = match &s.upstream {
        Some(u) => format!("+{}/-{}", u.ahead, u.behind),
        None => "-".to_string(),
    };
    let head = match &s.head {
        Some(h) => format!("{} {}", h.short_id, h.subject),
        None => "(no commits)".to_string(),
    };
    format!("{name:<24} {branch:<18} {dirty:<6} {tracking:<8} {head}")
}
