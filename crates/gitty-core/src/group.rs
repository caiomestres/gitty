use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::repository::{Repository, Workspace};

/// Name of the built-in default Group for Repositories without an assignment.
pub const UNGROUPED_GROUP_NAME: &str = "Ungrouped";

/// A hierarchical organizational category for Repositories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    /// `None` means this Group is at the top level of the tree.
    pub parent_id: Option<Uuid>,
}

/// One node in the Group hierarchy, with nested child Groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupTreeNode {
    pub group: Group,
    pub children: Vec<GroupTreeNode>,
}

impl Workspace {
    /// All Groups in the Workspace (flat list).
    pub fn list_groups(&self) -> &[Group] {
        &self.groups
    }

    /// Ensure the default "Ungrouped" top-level Group exists and return its id.
    pub fn ensure_ungrouped(&mut self) -> Uuid {
        if let Some(id) = self.ungrouped_group_id() {
            return id;
        }
        let id = Uuid::new_v4();
        self.groups.push(Group {
            id,
            name: UNGROUPED_GROUP_NAME.to_string(),
            parent_id: None,
        });
        id
    }

    /// Create a Group under `parent_id` (`None` for top-level). Returns the new id.
    pub fn create_group(&mut self, name: &str, parent_id: Option<Uuid>) -> Result<Uuid> {
        self.ensure_ungrouped();

        if let Some(pid) = parent_id {
            if self.find_group(pid).is_none() {
                return Err(CoreError::GroupNotFound(pid));
            }
        }

        if self.has_sibling_name(name, parent_id) {
            return Err(CoreError::DuplicateGroupName {
                name: name.to_string(),
                parent_id,
            });
        }

        let id = Uuid::new_v4();
        self.groups.push(Group {
            id,
            name: name.to_string(),
            parent_id,
        });
        Ok(id)
    }

    /// Rename an existing Group.
    pub fn rename_group(&mut self, id: Uuid, new_name: &str) -> Result<()> {
        let group = self
            .find_group(id)
            .ok_or(CoreError::GroupNotFound(id))?
            .clone();
        if group.name == UNGROUPED_GROUP_NAME && new_name != UNGROUPED_GROUP_NAME {
            return Err(CoreError::CannotModifyDefaultGroup(
                UNGROUPED_GROUP_NAME.into(),
            ));
        }
        if self.has_sibling_name_excluding(new_name, group.parent_id, id) {
            return Err(CoreError::DuplicateGroupName {
                name: new_name.to_string(),
                parent_id: group.parent_id,
            });
        }
        self.find_group_mut(id).expect("group exists").name = new_name.to_string();
        Ok(())
    }

    /// Delete a Group. Repositories in the Group move to Ungrouped; child Groups
    /// are re-parented to the deleted Group's parent.
    pub fn delete_group(&mut self, id: Uuid) -> Result<()> {
        let ungrouped_id = self.ensure_ungrouped();
        let group = self
            .find_group(id)
            .ok_or(CoreError::GroupNotFound(id))?
            .clone();
        if group.name == UNGROUPED_GROUP_NAME {
            return Err(CoreError::CannotDeleteDefaultGroup(
                UNGROUPED_GROUP_NAME.into(),
            ));
        }

        let parent_id = group.parent_id;
        for repo in &mut self.repositories {
            if repo.group_id == Some(id) {
                repo.group_id = Some(ungrouped_id);
            }
        }
        for child in self.groups.iter_mut().filter(|g| g.parent_id == Some(id)) {
            child.parent_id = parent_id;
        }
        self.groups.retain(|g| g.id != id);
        Ok(())
    }

    /// Move a Group under `new_parent_id` (`None` for top-level).
    pub fn move_group(&mut self, id: Uuid, new_parent_id: Option<Uuid>) -> Result<()> {
        self.ensure_ungrouped();
        if self.find_group(id).is_none() {
            return Err(CoreError::GroupNotFound(id));
        }
        if id == new_parent_id.unwrap_or(Uuid::nil()) {
            return Err(CoreError::CycleDetected { id });
        }
        if let Some(pid) = new_parent_id {
            if self.find_group(pid).is_none() {
                return Err(CoreError::GroupNotFound(pid));
            }
            if self.is_descendant_of(pid, id) {
                return Err(CoreError::CycleDetected { id });
            }
        }

        let parent_id = self.find_group(id).expect("group exists").parent_id;
        if self.has_sibling_name_excluding(
            &self.find_group(id).expect("group exists").name.clone(),
            new_parent_id,
            id,
        ) && parent_id != new_parent_id
        {
            let name = self.find_group(id).expect("group exists").name.clone();
            return Err(CoreError::DuplicateGroupName {
                name,
                parent_id: new_parent_id,
            });
        }

        self.find_group_mut(id).expect("group exists").parent_id = new_parent_id;
        Ok(())
    }

    /// Assign a Repository to a Group.
    pub fn assign_repo_to_group(&mut self, repo_id: Uuid, group_id: Uuid) -> Result<()> {
        self.ensure_ungrouped();
        if self.find_group(group_id).is_none() {
            return Err(CoreError::GroupNotFound(group_id));
        }
        let repo = self
            .find_repo_mut(repo_id)
            .ok_or(CoreError::RepositoryNotFound(repo_id))?;
        repo.group_id = Some(group_id);
        Ok(())
    }

    /// Repositories belonging to `group_id`.
    pub fn filter_by_group(&self, group_id: Uuid) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| self.effective_group_id(r) == group_id)
            .collect()
    }

    /// Nested tree of all top-level Groups.
    pub fn group_tree(&self) -> Vec<GroupTreeNode> {
        self.build_tree_nodes(None)
    }

    pub(crate) fn find_group(&self, id: Uuid) -> Option<&Group> {
        self.groups.iter().find(|g| g.id == id)
    }

    fn find_group_mut(&mut self, id: Uuid) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| g.id == id)
    }

    pub(crate) fn ungrouped_group_id(&self) -> Option<Uuid> {
        self.groups
            .iter()
            .find(|g| g.name == UNGROUPED_GROUP_NAME && g.parent_id.is_none())
            .map(|g| g.id)
    }

    pub(crate) fn effective_group_id(&self, repo: &Repository) -> Uuid {
        repo.group_id.unwrap_or_else(|| {
            self.ungrouped_group_id()
                .expect("Ungrouped group must exist after ensure_ungrouped")
        })
    }

    fn has_sibling_name(&self, name: &str, parent_id: Option<Uuid>) -> bool {
        self.groups
            .iter()
            .any(|g| g.parent_id == parent_id && g.name == name)
    }

    fn has_sibling_name_excluding(
        &self,
        name: &str,
        parent_id: Option<Uuid>,
        exclude_id: Uuid,
    ) -> bool {
        self.groups
            .iter()
            .any(|g| g.id != exclude_id && g.parent_id == parent_id && g.name == name)
    }

    fn is_descendant_of(&self, candidate: Uuid, ancestor: Uuid) -> bool {
        let mut current = Some(candidate);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.find_group(id).and_then(|g| g.parent_id);
        }
        false
    }

    fn build_tree_nodes(&self, parent_id: Option<Uuid>) -> Vec<GroupTreeNode> {
        self.groups
            .iter()
            .filter(|g| g.parent_id == parent_id)
            .map(|g| GroupTreeNode {
                group: g.clone(),
                children: self.build_tree_nodes(Some(g.id)),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::repository::Repository;

    #[test]
    fn ensure_ungrouped_creates_default_group() {
        let mut ws = Workspace::default();
        let id = ws.ensure_ungrouped();
        assert_eq!(ws.groups.len(), 1);
        assert_eq!(ws.groups[0].name, UNGROUPED_GROUP_NAME);
        assert_eq!(ws.groups[0].id, id);
    }

    #[test]
    fn create_group_rejects_duplicate_sibling_names() {
        let mut ws = Workspace::default();
        ws.create_group("Backend", None).unwrap();
        let err = ws.create_group("Backend", None).unwrap_err();
        assert!(matches!(err, CoreError::DuplicateGroupName { .. }));
    }

    #[test]
    fn create_nested_groups() {
        let mut ws = Workspace::default();
        let parent = ws.create_group("Work", None).unwrap();
        let child = ws.create_group("Client", Some(parent)).unwrap();
        assert_eq!(ws.groups.len(), 3);
        assert_eq!(ws.find_group(child).unwrap().parent_id, Some(parent));
    }

    #[test]
    fn delete_group_moves_repos_and_reparents_children() {
        let mut ws = Workspace::default();
        let ungrouped = ws.ensure_ungrouped();
        let parent = ws.create_group("Work", None).unwrap();
        let child = ws.create_group("Client", Some(parent)).unwrap();
        let mut repo = Repository::new(PathBuf::from("/a"), None);
        repo.group_id = Some(parent);
        ws.repositories.push(repo);

        ws.delete_group(parent).unwrap();
        assert!(ws.find_group(parent).is_none());
        assert_eq!(ws.find_group(child).unwrap().parent_id, None);
        assert_eq!(ws.repositories[0].group_id, Some(ungrouped));
    }

    #[test]
    fn cannot_delete_ungrouped() {
        let mut ws = Workspace::default();
        let id = ws.ensure_ungrouped();
        let err = ws.delete_group(id).unwrap_err();
        assert!(matches!(err, CoreError::CannotDeleteDefaultGroup(_)));
    }

    #[test]
    fn move_group_detects_cycles() {
        let mut ws = Workspace::default();
        let a = ws.create_group("A", None).unwrap();
        let b = ws.create_group("B", Some(a)).unwrap();
        let err = ws.move_group(a, Some(b)).unwrap_err();
        assert!(matches!(err, CoreError::CycleDetected { .. }));
    }

    #[test]
    fn assign_and_filter_by_group() {
        let mut ws = Workspace::default();
        let group = ws.create_group("Team", None).unwrap();
        let repo = Repository::new(PathBuf::from("/a"), None);
        let repo_id = repo.id;
        ws.repositories.push(repo);
        ws.assign_repo_to_group(repo_id, group).unwrap();
        assert_eq!(ws.filter_by_group(group).len(), 1);
    }

    #[test]
    fn group_tree_nests_children() {
        let mut ws = Workspace::default();
        ws.ensure_ungrouped();
        let parent = ws.create_group("Work", None).unwrap();
        ws.create_group("Client", Some(parent)).unwrap();
        let tree = ws.group_tree();
        let work = tree
            .iter()
            .find(|n| n.group.id == parent)
            .expect("Work node");
        assert_eq!(work.children.len(), 1);
    }
}
