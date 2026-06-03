mod commands;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use gitty_core::git::write::BatchOp;

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
    /// Show workspace and repository health status.
    Health {
        /// Show details for a single repository (path, name, or UUID).
        #[arg(long)]
        repo: Option<String>,
    },
    /// Manage the background scheduler.
    Scheduler {
        #[command(subcommand)]
        action: SchedulerAction,
    },
    /// Manage notification settings.
    Notification {
        #[command(subcommand)]
        action: NotificationAction,
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

#[derive(Subcommand)]
enum NotificationAction {
    /// Show current notification trigger configuration.
    Show,
    /// Set the notification trigger mode.
    Set {
        /// Trigger mode: on_critical, on_any_change, on_scheduler_complete, disabled
        mode: String,
    },
}

#[derive(Subcommand)]
enum SchedulerAction {
    /// Start the background scheduler.
    Start,
    /// Stop the background scheduler.
    Stop,
    /// Show scheduler status.
    Status,
    /// Internal: run the scheduler loop in foreground (used by Windows daemon).
    #[command(hide = true)]
    RunDaemon,
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

fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Scan { path } => commands::workspace::cmd_scan(&path),
        Commands::List => commands::workspace::cmd_list(),
        Commands::Status => commands::workspace::cmd_status(),
        Commands::Fetch { repo } => {
            commands::workspace::cmd_write(BatchOp::Fetch, "fetch", repo.as_deref())
        }
        Commands::Pull { repo } => {
            commands::workspace::cmd_write(BatchOp::Pull, "pull", repo.as_deref())
        }
        Commands::Checkout { branch, repo } => {
            commands::workspace::cmd_write(BatchOp::Checkout(&branch), "checkout", repo.as_deref())
        }
        Commands::Group { action } => commands::group::cmd_group(action),
        Commands::Tag { action } => commands::tag::cmd_tag(action),
        Commands::Filter { group, tag } => {
            commands::filter::cmd_filter(group.as_deref(), tag.as_deref())
        }
        Commands::Macro { action } => commands::macros::cmd_macro(action),
        Commands::Health { repo } => commands::health::cmd_health(repo.as_deref()),
        Commands::Scheduler { action } => commands::scheduler::cmd_scheduler(action),
        Commands::Notification { action } => commands::notification::cmd_notification(action),
    }
}
