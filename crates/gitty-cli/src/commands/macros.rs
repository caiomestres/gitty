use anyhow::{bail, Context, Result};
use gitty_core::config::Config;
use gitty_core::git::write::{match_repo, GitBinary};
use gitty_core::job::JobStatus;
use gitty_core::macro_def::{GitOp, ShellStep, Step, StepKind};
use gitty_core::Selection;

use super::resolve_group_id;
use crate::MacroAction;

pub fn cmd_macro(action: MacroAction) -> Result<()> {
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
                let repo_name = repo_ref.map(|r| r.display_name()).unwrap_or("<repo>");

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
) -> Result<Selection> {
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
