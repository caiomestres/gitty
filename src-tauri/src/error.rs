use gitty_core::CoreError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        let code = match &e {
            CoreError::GroupNotFound(_) => "group_not_found",
            CoreError::RepositoryNotFound(_) => "repository_not_found",
            CoreError::MacroNotFound(_) => "macro_not_found",
            CoreError::DuplicateGroupName { .. } => "duplicate_group_name",
            CoreError::DuplicateMacroName(_) => "duplicate_macro_name",
            CoreError::CannotDeleteDefaultGroup(_) => "cannot_delete_default_group",
            CoreError::CannotModifyDefaultGroup(_) => "cannot_modify_default_group",
            CoreError::CycleDetected { .. } => "cycle_detected",
            CoreError::EmptyTag => "empty_tag",
            CoreError::Io(_) | CoreError::PathNotFound(_) => "io_error",
            CoreError::Json(_) | CoreError::UnsupportedSchema { .. } | CoreError::NoConfigDir => {
                "config_error"
            }
            CoreError::Git(_) | CoreError::GitNotFound => "git_error",
            CoreError::LockContention { .. } => "lock_contention",
        };
        Self {
            code: code.to_string(),
            message: e.to_string(),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self {
            code: "unknown".to_string(),
            message: s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn core_error_maps_to_group_not_found() {
        let err = AppError::from(CoreError::GroupNotFound(Uuid::nil()));
        assert_eq!(err.code, "group_not_found");
    }

    #[test]
    fn core_error_maps_to_repository_not_found() {
        let err = AppError::from(CoreError::RepositoryNotFound(Uuid::nil()));
        assert_eq!(err.code, "repository_not_found");
    }

    #[test]
    fn core_error_maps_to_macro_not_found() {
        let err = AppError::from(CoreError::MacroNotFound("test".into()));
        assert_eq!(err.code, "macro_not_found");
    }

    #[test]
    fn core_error_maps_to_duplicate_group_name() {
        let err = AppError::from(CoreError::DuplicateGroupName {
            name: "test".into(),
            parent_id: None,
        });
        assert_eq!(err.code, "duplicate_group_name");
    }

    #[test]
    fn core_error_maps_to_cannot_delete_default_group() {
        let err = AppError::from(CoreError::CannotDeleteDefaultGroup("Ungrouped".into()));
        assert_eq!(err.code, "cannot_delete_default_group");
    }

    #[test]
    fn core_error_maps_to_cycle_detected() {
        let err = AppError::from(CoreError::CycleDetected { id: Uuid::nil() });
        assert_eq!(err.code, "cycle_detected");
    }

    #[test]
    fn core_error_maps_to_empty_tag() {
        let err = AppError::from(CoreError::EmptyTag);
        assert_eq!(err.code, "empty_tag");
    }

    #[test]
    fn string_maps_to_unknown() {
        let err = AppError::from("something went wrong".to_string());
        assert_eq!(err.code, "unknown");
        assert_eq!(err.message, "something went wrong");
    }
}
