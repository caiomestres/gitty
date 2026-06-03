use anyhow::{Context, Result};
use gitty_core::config::Config;
use gitty_core::git::write::match_repo;

use crate::TagAction;

pub fn cmd_tag(action: TagAction) -> Result<()> {
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
