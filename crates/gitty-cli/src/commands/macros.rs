use anyhow::{bail, Context, Result};
use gitty_core::config::Config;
use gitty_core::git::write::{match_repo, GitBinary};
use gitty_core::job::JobStatus;
use gitty_core::macro_def::{GitOp, RetryConfig, ShellStep, Step, StepKind};
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
                let retry = step
                    .retry
                    .as_ref()
                    .map(|r| format!(" [retry={}:backoff={}]", r.max_attempts, r.backoff_seconds))
                    .unwrap_or_default();
                println!("  {}. {desc}{cond}{retry}{confirm}", i + 1);
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
    let (kind, retry) = if arg == "fetch" || arg == "pull" {
        let op = if arg == "fetch" {
            GitOp::Fetch
        } else {
            GitOp::Pull
        };
        (StepKind::GitOp(op), None)
    } else if let Some(rest) = arg.strip_prefix("fetch:") {
        let (retry, _) = parse_retry_suffix(rest.split(':').collect())?;
        (StepKind::GitOp(GitOp::Fetch), retry)
    } else if let Some(rest) = arg.strip_prefix("pull:") {
        let (retry, _) = parse_retry_suffix(rest.split(':').collect())?;
        (StepKind::GitOp(GitOp::Pull), retry)
    } else if let Some(rest) = arg.strip_prefix("checkout:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.is_empty() {
            bail!("checkout requires a branch name");
        }
        let (retry, remaining) = parse_retry_suffix(parts)?;
        if remaining.is_empty() {
            bail!("checkout requires a branch name");
        }
        let branch = remaining.join(":");
        (StepKind::GitOp(GitOp::Checkout { branch }), retry)
    } else if let Some(rest) = arg.strip_prefix("shell:") {
        let parts: Vec<&str> = rest.split(':').collect();
        let (retry, remaining) = parse_retry_suffix(parts)?;
        if let Some(retry_config) = retry {
            eprintln!(
                "warning: retry settings (max_attempts={}, backoff_seconds={}) are ignored for shell steps",
                retry_config.max_attempts, retry_config.backoff_seconds
            );
        }
        let command = if remaining.is_empty() {
            bail!("shell requires a command");
        } else {
            remaining.join(":")
        };
        (
            StepKind::Shell(ShellStep {
                command,
                label: None,
            }),
            None,
        )
    } else {
        bail!("unrecognized step '{arg}'. Use: fetch, pull, checkout:<branch>, or shell:<command>");
    };

    Ok(Step {
        kind,
        condition: None,
        rollback: None,
        confirm: false,
        retry,
    })
}

fn parse_retry_suffix(parts: Vec<&str>) -> Result<(Option<RetryConfig>, Vec<&str>)> {
    let mut max_attempts = None;
    let mut backoff_seconds = None;
    let mut end = parts.len();

    while end > 0 {
        let part = parts[end - 1];
        if let Some(value) = part.strip_prefix("retry=") {
            max_attempts = Some(
                value
                    .parse::<u32>()
                    .with_context(|| format!("invalid retry value '{value}'"))?,
            );
            end -= 1;
        } else if let Some(value) = part.strip_prefix("backoff=") {
            backoff_seconds = Some(
                value
                    .parse::<u64>()
                    .with_context(|| format!("invalid backoff value '{value}'"))?,
            );
            end -= 1;
        } else {
            break;
        }
    }

    let retry = max_attempts.map(|attempts| RetryConfig {
        max_attempts: attempts,
        backoff_seconds: backoff_seconds.unwrap_or(2),
    });

    Ok((retry, parts[..end].to_vec()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fetch_with_retry() {
        let step = parse_step_arg("fetch:retry=3").unwrap();
        assert!(matches!(step.kind, StepKind::GitOp(GitOp::Fetch)));
        let retry = step.retry.unwrap();
        assert_eq!(retry.max_attempts, 3);
        assert_eq!(retry.backoff_seconds, 2);
    }

    #[test]
    fn parse_pull_with_retry_and_backoff() {
        let step = parse_step_arg("pull:retry=3:backoff=5").unwrap();
        assert!(matches!(step.kind, StepKind::GitOp(GitOp::Pull)));
        let retry = step.retry.unwrap();
        assert_eq!(retry.max_attempts, 3);
        assert_eq!(retry.backoff_seconds, 5);
    }

    #[test]
    fn parse_fetch_without_retry_is_backward_compatible() {
        let step = parse_step_arg("fetch").unwrap();
        assert!(matches!(step.kind, StepKind::GitOp(GitOp::Fetch)));
        assert!(step.retry.is_none());
    }

    #[test]
    fn parse_shell_ignores_retry_params() {
        let step = parse_step_arg("shell:echo hello:retry=3").unwrap();
        assert!(matches!(step.kind, StepKind::Shell(_)));
        assert!(step.retry.is_none());
        if let StepKind::Shell(shell) = step.kind {
            assert_eq!(shell.command, "echo hello");
        }
    }

    #[test]
    fn format_step_kind_includes_retry_in_show_output() {
        let step = parse_step_arg("fetch:retry=3").unwrap();
        let desc = format_step_kind(&step.kind);
        assert_eq!(desc, "git fetch --all");
        let retry = step
            .retry
            .as_ref()
            .map(|r| format!(" [retry={}:backoff={}]", r.max_attempts, r.backoff_seconds))
            .unwrap_or_default();
        assert_eq!(retry, " [retry=3:backoff=2]");
    }

    #[test]
    fn parse_checkout_with_colon_in_branch_name() {
        let step = parse_step_arg("checkout:refs:heads/main:retry=2").unwrap();
        if let StepKind::GitOp(GitOp::Checkout { branch }) = &step.kind {
            assert_eq!(branch, "refs:heads/main");
        } else {
            panic!("expected checkout");
        }
        assert_eq!(step.retry.unwrap().max_attempts, 2);
    }

    #[test]
    fn parse_shell_with_colons_in_command() {
        let step = parse_step_arg("shell:docker run -p 8080:80 nginx:latest:retry=3").unwrap();
        assert!(step.retry.is_none());
        if let StepKind::Shell(shell) = step.kind {
            assert_eq!(shell.command, "docker run -p 8080:80 nginx:latest");
        } else {
            panic!("expected shell");
        }
    }
}
