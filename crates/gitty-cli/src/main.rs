use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use gitty_core::config::Config;
use gitty_core::git::read::{self, RepositoryStatus};
use gitty_core::git::write::{match_repo, BatchOp, BatchResult, GitBinary, RepoOutcome};
use gitty_core::job::JobStatus;
use gitty_core::macro_def::{GitOp, ShellStep, Step, StepKind};
use gitty_core::repository::RepositoryState;
use gitty_core::scan_and_reconcile;
use uuid::Uuid;

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
    /// Fetch all remotes for every registered repository (or a single one).
    Fetch {
        /// Optional repository path or directory name to target.
        repo: Option<String>,
    },
    /// Pull every registered repository (or a single one).
    Pull {
        /// Optional repository path or directory name to target.
        repo: Option<String>,
    },
    /// Checkout a branch in every registered repository (or a single one).
    Checkout {
        /// The branch name to check out.
        branch: String,
        /// Optional repository path or directory name to target.
        #[arg(long)]
        repo: Option<String>,
    },
    /// Manage Groups.
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },
    /// Manage Tags.
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },
    /// Filter repositories by Group or Tag.
    Filter {
        /// Filter by Group name or id.
        #[arg(long)]
        group: Option<String>,
        /// Filter by Tag name.
        #[arg(long)]
        tag: Option<String>,
    },
    /// Manage and run Macros.
    Macro {
        #[command(subcommand)]
        action: MacroAction,
    },
}

#[derive(Subcommand)]
enum GroupAction {
    /// List all Groups.
    List,
    /// Create a new Group.
    Create {
        /// Name for the new Group.
        name: String,
        /// Parent Group name or id (omit for top-level).
        #[arg(long)]
        parent: Option<String>,
    },
    /// Rename an existing Group.
    Rename {
        /// Current name or id of the Group.
        group: String,
        /// New name for the Group.
        name: String,
    },
    /// Delete a Group (repos move to Ungrouped).
    Delete {
        /// Name or id of the Group to delete.
        group: String,
    },
    /// Assign a Repository to a Group.
    Assign {
        /// Repository path or directory name.
        repo: String,
        /// Group name or id.
        group: String,
    },
    /// Show the Group tree hierarchy.
    Tree,
}

#[derive(Subcommand)]
enum TagAction {
    /// List all Tags in use.
    List,
    /// Add a Tag to a Repository.
    Add {
        /// Repository path or directory name.
        repo: String,
        /// Tag to add.
        tag: String,
    },
    /// Remove a Tag from a Repository.
    Remove {
        /// Repository path or directory name.
        repo: String,
        /// Tag to remove.
        tag: String,
    },
}

#[derive(Subcommand)]
enum MacroAction {
    /// List all defined Macros.
    List,
    /// Define a new Macro from inline steps.
    Define {
        /// Name for the new Macro.
        name: String,
        /// Steps: "fetch", "pull", "checkout:branch", or "shell:command".
        #[arg(required = true, num_args = 1..)]
        steps: Vec<String>,
    },
    /// Delete a Macro.
    Delete {
        /// Name or id of the Macro.
        name: String,
    },
    /// Show steps of a Macro.
    Show {
        /// Name or id of the Macro.
        name: String,
    },
    /// Run a Macro against a selection of repositories.
    Run {
        /// Name or id of the Macro.
        name: String,
        /// Optional: filter by Group.
        #[arg(long)]
        group: Option<String>,
        /// Optional: filter by Tag.
        #[arg(long)]
        tag: Option<String>,
        /// Optional: target a single repository.
        #[arg(long)]
        repo: Option<String>,
    },
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
        Commands::Fetch { repo } => cmd_write(BatchOp::Fetch, "fetch", repo.as_deref()),
        Commands::Pull { repo } => cmd_write(BatchOp::Pull, "pull", repo.as_deref()),
        Commands::Checkout { branch, repo } => {
            cmd_write(BatchOp::Checkout(&branch), "checkout", repo.as_deref())
        }
        Commands::Group { action } => cmd_group(action),
        Commands::Tag { action } => cmd_tag(action),
        Commands::Filter { group, tag } => cmd_filter(group.as_deref(), tag.as_deref()),
        Commands::Macro { action } => cmd_macro(action),
    }
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

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

fn cmd_write(op: BatchOp<'_>, label: &str, target: Option<&str>) -> Result<()> {
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

// ---------------------------------------------------------------------------
// Group commands
// ---------------------------------------------------------------------------

fn cmd_group(action: GroupAction) -> Result<()> {
    match action {
        GroupAction::List => {
            let config = Config::load().context("loading config")?;
            let groups = config.workspace.list_groups();
            if groups.is_empty() {
                println!("No groups defined.");
                return Ok(());
            }
            for g in groups {
                let parent = match g.parent_id {
                    Some(pid) => config
                        .workspace
                        .list_groups()
                        .iter()
                        .find(|p| p.id == pid)
                        .map(|p| p.name.as_str())
                        .unwrap_or("?"),
                    None => "-",
                };
                let count = config.workspace.filter_by_group(g.id).len();
                println!("{:<24} parent: {:<16} repos: {}", g.name, parent, count);
            }
        }
        GroupAction::Create { name, parent } => {
            let mut config = Config::load().context("loading config")?;
            let parent_id = parent
                .as_deref()
                .map(|p| resolve_group_id(&config, p))
                .transpose()?;
            let id = config
                .workspace
                .create_group(&name, parent_id)
                .context("creating group")?;
            config.save().context("saving config")?;
            println!("Created group '{name}' ({id})");
        }
        GroupAction::Rename { group, name } => {
            let mut config = Config::load().context("loading config")?;
            let id = resolve_group_id(&config, &group)?;
            config
                .workspace
                .rename_group(id, &name)
                .context("renaming group")?;
            config.save().context("saving config")?;
            println!("Renamed group to '{name}'");
        }
        GroupAction::Delete { group } => {
            let mut config = Config::load().context("loading config")?;
            let id = resolve_group_id(&config, &group)?;
            let name = config
                .workspace
                .list_groups()
                .iter()
                .find(|g| g.id == id)
                .map(|g| g.name.clone())
                .unwrap_or_default();
            config
                .workspace
                .delete_group(id)
                .context("deleting group")?;
            config.save().context("saving config")?;
            println!("Deleted group '{name}'");
        }
        GroupAction::Assign { repo, group } => {
            let mut config = Config::load().context("loading config")?;
            let repo_entry = match_repo(&config.workspace.repositories, &repo)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let repo_id = repo_entry.id;
            let group_id = resolve_group_id(&config, &group)?;
            config
                .workspace
                .assign_repo_to_group(repo_id, group_id)
                .context("assigning repo to group")?;
            config.save().context("saving config")?;
            let group_name = config
                .workspace
                .list_groups()
                .iter()
                .find(|g| g.id == group_id)
                .map(|g| g.name.as_str())
                .unwrap_or("?");
            println!("Assigned '{repo}' to group '{group_name}'");
        }
        GroupAction::Tree => {
            let mut config = Config::load().context("loading config")?;
            config.workspace.ensure_ungrouped();
            let tree = config.workspace.group_tree();
            if tree.is_empty() {
                println!("No groups defined.");
                return Ok(());
            }
            for node in &tree {
                print_tree_node(node, 0, &config);
            }
        }
    }
    Ok(())
}

fn print_tree_node(node: &gitty_core::GroupTreeNode, depth: usize, config: &Config) {
    let indent = "  ".repeat(depth);
    let count = config.workspace.filter_by_group(node.group.id).len();
    println!("{indent}{} ({count} repos)", node.group.name);
    for child in &node.children {
        print_tree_node(child, depth + 1, config);
    }
}

fn resolve_group_id(config: &Config, name_or_id: &str) -> Result<Uuid> {
    if let Ok(id) = Uuid::parse_str(name_or_id) {
        if config.workspace.list_groups().iter().any(|g| g.id == id) {
            return Ok(id);
        }
    }
    config
        .workspace
        .list_groups()
        .iter()
        .find(|g| g.name == name_or_id)
        .map(|g| g.id)
        .ok_or_else(|| anyhow::anyhow!("no group matching '{name_or_id}'"))
}

// ---------------------------------------------------------------------------
// Tag commands
// ---------------------------------------------------------------------------

fn cmd_tag(action: TagAction) -> Result<()> {
    match action {
        TagAction::List => {
            let config = Config::load().context("loading config")?;
            let tags = config.workspace.list_all_tags();
            if tags.is_empty() {
                println!("No tags in use.");
                return Ok(());
            }
            for tag in &tags {
                let count = config.workspace.filter_by_tag(tag).len();
                println!("{tag:<24} {count} repos");
            }
        }
        TagAction::Add { repo, tag } => {
            let mut config = Config::load().context("loading config")?;
            let repo_entry = match_repo(&config.workspace.repositories, &repo)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let repo_id = repo_entry.id;
            config
                .workspace
                .add_tag(repo_id, &tag)
                .context("adding tag")?;
            config.save().context("saving config")?;
            println!("Tagged '{repo}' with '{tag}'");
        }
        TagAction::Remove { repo, tag } => {
            let mut config = Config::load().context("loading config")?;
            let repo_entry = match_repo(&config.workspace.repositories, &repo)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let repo_id = repo_entry.id;
            config
                .workspace
                .remove_tag(repo_id, &tag)
                .context("removing tag")?;
            config.save().context("saving config")?;
            println!("Removed tag '{tag}' from '{repo}'");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Filter command
// ---------------------------------------------------------------------------

fn cmd_filter(group: Option<&str>, tag: Option<&str>) -> Result<()> {
    if group.is_none() && tag.is_none() {
        bail!("at least one of --group or --tag is required");
    }
    let config = Config::load().context("loading config")?;

    let mut repos: Vec<_> = config.workspace.repositories.iter().collect();

    if let Some(group_name) = group {
        let group_id = resolve_group_id(&config, group_name)?;
        let group_repos = config.workspace.filter_by_group(group_id);
        repos.retain(|r| group_repos.iter().any(|gr| gr.id == r.id));
    }

    if let Some(tag_name) = tag {
        let tag_repos = config.workspace.filter_by_tag(tag_name);
        repos.retain(|r| tag_repos.iter().any(|tr| tr.id == r.id));
    }

    if repos.is_empty() {
        println!("No repositories match the filter.");
        return Ok(());
    }

    for repo in &repos {
        let name = repo
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<repo>");
        let tags = if repo.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", repo.tags.join(", "))
        };
        println!("{name:<24} {}{tags}", repo.path.display());
    }
    println!("\n{} repositories", repos.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Macro commands
// ---------------------------------------------------------------------------

fn cmd_macro(action: MacroAction) -> Result<()> {
    match action {
        MacroAction::List => {
            let config = Config::load().context("loading config")?;
            let macros = config.workspace.list_macros();
            if macros.is_empty() {
                println!("No macros defined. Use `gitty macro define` to create one.");
                return Ok(());
            }
            for m in macros {
                let step_count = m.steps.len();
                let step_word = if step_count == 1 { "step" } else { "steps" };
                println!("{:<24} {} {step_word}", m.name, step_count);
            }
        }
        MacroAction::Define { name, steps } => {
            let parsed_steps = steps
                .iter()
                .map(|s| parse_step_arg(s))
                .collect::<Result<Vec<_>>>()?;
            let mut config = Config::load().context("loading config")?;
            let id = config
                .workspace
                .define_macro(&name, parsed_steps)
                .context("defining macro")?;
            config.save().context("saving config")?;
            println!("Defined macro '{name}' ({id}) with {} steps", steps.len());
        }
        MacroAction::Delete { name } => {
            let mut config = Config::load().context("loading config")?;
            let macro_def = config
                .workspace
                .find_macro(&name)
                .ok_or_else(|| anyhow::anyhow!("no macro matching '{name}'"))?;
            let id = macro_def.id;
            let actual_name = macro_def.name.clone();
            config
                .workspace
                .delete_macro(id)
                .context("deleting macro")?;
            config.save().context("saving config")?;
            println!("Deleted macro '{actual_name}'");
        }
        MacroAction::Show { name } => {
            let config = Config::load().context("loading config")?;
            let macro_def = config
                .workspace
                .find_macro(&name)
                .ok_or_else(|| anyhow::anyhow!("no macro matching '{name}'"))?;
            println!("Macro: {} ({})", macro_def.name, macro_def.id);
            println!("Steps:");
            for (i, step) in macro_def.steps.iter().enumerate() {
                let desc = format_step_kind(&step.kind);
                let cond = step
                    .condition
                    .as_deref()
                    .map(|c| format!(" [if {c}]"))
                    .unwrap_or_default();
                let confirm = if step.confirm { " (confirm)" } else { "" };
                println!("  {}. {desc}{cond}{confirm}", i + 1);
            }
        }
        MacroAction::Run {
            name,
            group,
            tag,
            repo,
        } => {
            let config = Config::load().context("loading config")?;
            let macro_def = config
                .workspace
                .find_macro(&name)
                .ok_or_else(|| anyhow::anyhow!("no macro matching '{name}'"))?
                .clone();

            let selection =
                resolve_selection(&config, group.as_deref(), tag.as_deref(), repo.as_deref())?;
            let repos = selection.resolve(&config.workspace);

            if repos.is_empty() {
                println!("No repositories match the selection.");
                return Ok(());
            }

            let git = GitBinary::resolve().context("locating git")?;
            println!(
                "Running macro '{}' on {} repositories...\n",
                macro_def.name,
                repos.len()
            );

            let jobs = gitty_core::execute_macro(&macro_def, &repos, &git);

            let mut success = 0usize;
            let mut failed = 0usize;
            let mut skipped = 0usize;

            for job in &jobs {
                let repo_ref = config.workspace.find_by_id(job.repo_id);
                let repo_name = repo_ref
                    .and_then(|r| r.path.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("<repo>");

                match &job.status {
                    JobStatus::Success => {
                        println!("\u{2713} {repo_name:<22} all steps succeeded");
                        success += 1;
                    }
                    JobStatus::Failed { error } => {
                        println!("\u{2717} {repo_name:<22} {error}");
                        failed += 1;
                    }
                    JobStatus::Skipped { reason } => {
                        println!("\u{2298} {repo_name:<22} [skipped] {reason}");
                        skipped += 1;
                    }
                    _ => {}
                }
            }

            println!(
                "\n{}: {} ok, {} failed, {} skipped",
                macro_def.name, success, failed, skipped
            );
        }
    }
    Ok(())
}

fn parse_step_arg(arg: &str) -> Result<Step> {
    let kind = if arg == "fetch" {
        StepKind::GitOp(GitOp::Fetch)
    } else if arg == "pull" {
        StepKind::GitOp(GitOp::Pull)
    } else if let Some(branch) = arg.strip_prefix("checkout:") {
        StepKind::GitOp(GitOp::Checkout {
            branch: branch.to_string(),
        })
    } else if let Some(cmd) = arg.strip_prefix("shell:") {
        StepKind::Shell(ShellStep {
            command: cmd.to_string(),
            label: None,
        })
    } else {
        bail!("unrecognized step '{arg}'. Use: fetch, pull, checkout:<branch>, or shell:<command>");
    };

    Ok(Step {
        kind,
        condition: None,
        rollback: None,
        confirm: false,
    })
}

fn format_step_kind(kind: &StepKind) -> String {
    match kind {
        StepKind::GitOp(GitOp::Fetch) => "git fetch --all".to_string(),
        StepKind::GitOp(GitOp::Pull) => "git pull".to_string(),
        StepKind::GitOp(GitOp::Checkout { branch }) => format!("git checkout {branch}"),
        StepKind::Shell(shell) => {
            let label = shell.label.as_deref().unwrap_or(&shell.command);
            format!("shell: {label}")
        }
    }
}

fn resolve_selection(
    config: &Config,
    group: Option<&str>,
    tag: Option<&str>,
    repo: Option<&str>,
) -> Result<gitty_core::Selection> {
    use gitty_core::Selection;

    if let Some(repo_target) = repo {
        let matched = match_repo(&config.workspace.repositories, repo_target)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        return Ok(Selection::Single(matched.id));
    }
    if let Some(group_name) = group {
        let group_id = resolve_group_id(config, group_name)?;
        return Ok(Selection::Group(group_id));
    }
    if let Some(tag_name) = tag {
        return Ok(Selection::Tag(tag_name.to_string()));
    }
    Ok(Selection::All)
}

// ---------------------------------------------------------------------------
// Shared formatting
// ---------------------------------------------------------------------------

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
