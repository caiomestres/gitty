use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::{Repository, Workspace};

/// Unified target selection for Macro execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    All,
    Single(Uuid),
    Group(Uuid),
    Tag(String),
    Multiple(Vec<Uuid>),
}

impl Selection {
    /// Resolve this selection to concrete Repositories in `workspace`.
    pub fn resolve<'a>(&self, workspace: &'a Workspace) -> Vec<&'a Repository> {
        match self {
            Self::All => workspace.repositories.iter().collect(),
            Self::Single(id) => workspace.find_by_id(*id).into_iter().collect(),
            Self::Group(id) => workspace.filter_by_group(*id),
            Self::Tag(tag) => workspace.filter_by_tag(tag),
            Self::Multiple(ids) => ids
                .iter()
                .filter_map(|id| workspace.find_by_id(*id))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::repository::Repository;

    #[test]
    fn resolve_all_and_single() {
        let mut ws = Workspace::default();
        let repo = Repository::new(PathBuf::from("/a"), None);
        let id = repo.id;
        ws.repositories.push(repo);
        assert_eq!(Selection::All.resolve(&ws).len(), 1);
        assert_eq!(Selection::Single(id).resolve(&ws).len(), 1);
    }

    #[test]
    fn resolve_group_and_tag() {
        let mut ws = Workspace::default();
        let group = ws.create_group("G", None).unwrap();
        let mut repo = Repository::new(PathBuf::from("/a"), None);
        repo.group_id = Some(group);
        repo.tags = vec!["hot".into()];
        ws.repositories.push(repo);
        assert_eq!(Selection::Group(group).resolve(&ws).len(), 1);
        assert_eq!(Selection::Tag("hot".into()).resolve(&ws).len(), 1);
    }
}
