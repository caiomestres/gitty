use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use gitty_core::config::Config;
use gitty_core::git::read::{self, RepositoryStatus};
use gitty_core::repository::RepositoryState;
use gitty_core::scan_and_reconcile;

#[derive(Parser)]
#[command(name = "gitty", version)]
#[command(about = "Workspace synchronization and orchestration for Git repositories")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a directory for Git repositories and register them.
    Scan {
        /// Directory to scan recursively for `.git` repositories.
        path: PathBuf,
    },
    /// List all registered repositories.
    List,
    /// Show the Git status of each registered repository.
    Status,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Commands::Scan { path } => cmd_scan(&path),
        Commands::List => cmd_list(),
        Commands::Status => cmd_status(),
    }
}

fn cmd_scan(path: &Path) -> Result<()> {
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

fn cmd_list() -> Result<()> {
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

fn cmd_status() -> Result<()> {
    let config = Config::load().context("loading config")?;
    let repos = &config.workspace.repositories;
    if repos.is_empty() {
        println!("No repositories tracked yet. Run `gitty scan <path>` to discover some.");
        return Ok(());
    }
    for repo in repos {
        let name = repo
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<repo>");

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
