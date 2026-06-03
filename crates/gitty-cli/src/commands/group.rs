use anyhow::Context;
use anyhow::Result;
use gitty_core::config::Config;
use gitty_core::git::write::match_repo;

use super::resolve_group_id;
use crate::GroupAction;

pub fn cmd_group(action: GroupAction) -> Result<()> {
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
