use gitty_core::CoreError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

fn hint_for_core_error(e: &CoreError) -> Option<String> {
    match e {
        CoreError::GitNotFound => Some(
            "Install Git and ensure it is in your PATH, then restart Gitty.".into(),
        ),
        CoreError::LockContention { .. } => Some(
            "Another process is using this Repository. Wait for it to finish or check for stale locks."
                .into(),
        ),
        CoreError::UnsupportedSchema { .. } => Some(
            "This config was created by a newer version of Gitty. Update Gitty to the latest version."
                .into(),
        ),
        CoreError::PathNotFound(_) => Some(
            "The path does not exist on disk. Check that it hasn't been moved or deleted.".into(),
        ),
        CoreError::Io(io_err) if io_err.to_string().to_lowercase().contains("permission denied") => {
            Some(
                "Permission denied. Check file permissions or run with elevated privileges.".into(),
            )
        }
        CoreError::NoConfigDir => Some(
            "Could not find a config directory. Check your OS user profile.".into(),
        ),
        _ => None,
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        let hint = hint_for_core_error(&e);
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
            CoreError::Other(_) => "internal_error",
        };
        Self {
            code: code.to_string(),
            message: e.to_string(),
            hint,
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self {
            code: "unknown".to_string(),
            message: s,
            hint: None,
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
        assert_eq!(err.hint, None);
    }

    #[test]
    fn core_error_without_hint_has_none() {
        let err = AppError::from(CoreError::GroupNotFound(Uuid::nil()));
        assert_eq!(err.hint, None);
    }

    #[test]
    fn git_not_found_hint() {
        let err = AppError::from(CoreError::GitNotFound);
        assert_eq!(
            err.hint.as_deref(),
            Some("Install Git and ensure it is in your PATH, then restart Gitty.")
        );
    }

    #[test]
    fn lock_contention_hint() {
        let err = AppError::from(CoreError::LockContention {
            repo_id: Uuid::nil(),
            pid: 1234,
            since: "2026-01-01".into(),
        });
        assert_eq!(
            err.hint.as_deref(),
            Some("Another process is using this Repository. Wait for it to finish or check for stale locks.")
        );
    }

    #[test]
    fn unsupported_schema_hint() {
        let err = AppError::from(CoreError::UnsupportedSchema {
            found: 99,
            expected: 1,
        });
        assert_eq!(
            err.hint.as_deref(),
            Some("This config was created by a newer version of Gitty. Update Gitty to the latest version.")
        );
    }

    #[test]
    fn path_not_found_hint() {
        let err = AppError::from(CoreError::PathNotFound("/missing".into()));
        assert_eq!(
            err.hint.as_deref(),
            Some("The path does not exist on disk. Check that it hasn't been moved or deleted.")
        );
    }

    #[test]
    fn io_permission_denied_hint() {
        let err = AppError::from(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        )));
        assert_eq!(
            err.hint.as_deref(),
            Some("Permission denied. Check file permissions or run with elevated privileges.")
        );
    }

    #[test]
    fn io_without_permission_denied_has_no_hint() {
        let err = AppError::from(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        )));
        assert_eq!(err.hint, None);
    }

    #[test]
    fn no_config_dir_hint() {
        let err = AppError::from(CoreError::NoConfigDir);
        assert_eq!(
            err.hint.as_deref(),
            Some("Could not find a config directory. Check your OS user profile.")
        );
    }
}
