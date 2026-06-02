use crate::error::{CoreError, Result};
use crate::repository::{Repository, Workspace};

/// Built-in system Tag for favorited Repositories.
pub const FAVORITE_TAG: &str = "Favorite";

impl Workspace {
    /// Attach a Tag to a Repository (idempotent for duplicate tags).
    pub fn add_tag(&mut self, repo_id: uuid::Uuid, tag: &str) -> Result<()> {
        let tag = tag.trim();
        if tag.is_empty() {
            return Err(CoreError::EmptyTag);
        }
        let repo = self
            .find_repo_mut(repo_id)
            .ok_or(CoreError::RepositoryNotFound(repo_id))?;
        if !repo.tags.iter().any(|t| t == tag) {
            repo.tags.push(tag.to_string());
        }
        Ok(())
    }

    /// Remove a Tag from a Repository.
    pub fn remove_tag(&mut self, repo_id: uuid::Uuid, tag: &str) -> Result<()> {
        let repo = self
            .find_repo_mut(repo_id)
            .ok_or(CoreError::RepositoryNotFound(repo_id))?;
        repo.tags.retain(|t| t != tag);
        Ok(())
    }

    /// Distinct Tags across all Repositories, sorted lexicographically.
    pub fn list_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .repositories
            .iter()
            .flat_map(|r| r.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Repositories that carry `tag`.
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::repository::Repository;

    #[test]
    fn add_and_remove_tag() {
        let mut ws = Workspace::default();
        let repo = Repository::new(PathBuf::from("/a"), None);
        let id = repo.id;
        ws.repositories.push(repo);

        ws.add_tag(id, FAVORITE_TAG).unwrap();
        assert_eq!(ws.repositories[0].tags, vec![FAVORITE_TAG]);

        ws.remove_tag(id, FAVORITE_TAG).unwrap();
        assert!(ws.repositories[0].tags.is_empty());
    }

    #[test]
    fn add_tag_rejects_empty() {
        let mut ws = Workspace::default();
        let repo = Repository::new(PathBuf::from("/a"), None);
        let id = repo.id;
        ws.repositories.push(repo);
        assert!(matches!(
            ws.add_tag(id, "  ").unwrap_err(),
            CoreError::EmptyTag
        ));
    }

    #[test]
    fn list_all_tags_is_distinct_and_sorted() {
        let mut ws = Workspace::default();
        let mut a = Repository::new(PathBuf::from("/a"), None);
        a.tags = vec!["beta".into(), "alpha".into()];
        let mut b = Repository::new(PathBuf::from("/b"), None);
        b.tags = vec!["alpha".into()];
        ws.repositories.extend([a, b]);
        assert_eq!(ws.list_all_tags(), vec!["alpha", "beta"]);
    }

    #[test]
    fn filter_by_tag_returns_matching_repos() {
        let mut ws = Workspace::default();
        let mut tagged = Repository::new(PathBuf::from("/a"), None);
        tagged.tags = vec!["deploy".into()];
        let plain = Repository::new(PathBuf::from("/b"), None);
        ws.repositories.extend([tagged, plain]);
        assert_eq!(ws.filter_by_tag("deploy").len(), 1);
    }
}
