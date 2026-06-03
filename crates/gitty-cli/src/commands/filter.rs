use anyhow::{bail, Context, Result};
use gitty_core::config::Config;

use super::resolve_group_id;

pub fn cmd_filter(group: Option<&str>, tag: Option<&str>) -> Result<()> {
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
        let name = repo.display_name();
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
