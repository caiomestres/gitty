//! Error classification for failed `git` operations (D14).

use std::fmt;

/// Actionable category for a failed `git` operation, derived from stderr
/// pattern matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Network,
    Auth,
    Conflict,
    DirtyWorkTree,
    BranchNotFound,
    NoUpstream,
    Unknown(String),
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => write!(f, "network error"),
            Self::Auth => write!(f, "authentication failed"),
            Self::Conflict => write!(f, "merge conflict"),
            Self::DirtyWorkTree => write!(f, "dirty working tree"),
            Self::BranchNotFound => write!(f, "branch not found"),
            Self::NoUpstream => write!(f, "no upstream configured"),
            Self::Unknown(msg) => write!(f, "{msg}"),
        }
    }
}

/// Classify stderr from a failed `git` command into an actionable category.
/// Case-insensitive substring matching, first match wins.
pub fn classify_error(stderr: &str) -> ErrorCategory {
    let lower = stderr.to_lowercase();

    if lower.contains("could not resolve host")
        || lower.contains("unable to access")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
        || lower.contains("timed out")
        || lower.contains("connection timed out")
    {
        return ErrorCategory::Network;
    }
    if lower.contains("authentication failed")
        || lower.contains("invalid username or password")
        || lower.contains("could not read username")
        || lower.contains("could not read from remote")
        || lower.contains("terminal prompts disabled")
    {
        return ErrorCategory::Auth;
    }
    if lower.contains("conflict") && lower.contains("merge")
        || lower.contains("fix conflicts")
        || lower.contains("automatic merge failed")
    {
        return ErrorCategory::Conflict;
    }
    if lower.contains("your local changes")
        || lower.contains("please commit your changes or stash them")
        || lower.contains("overwritten by")
    {
        return ErrorCategory::DirtyWorkTree;
    }
    if lower.contains("did not match any")
        || lower.contains("pathspec")
            && (lower.contains("did not match") || lower.contains("unknown revision"))
        || lower.contains("not a valid branch name")
        || lower.contains("invalid reference")
    {
        return ErrorCategory::BranchNotFound;
    }
    if lower.contains("no tracking information")
        || lower.contains("no upstream")
        || lower.contains("there is no tracking information")
    {
        return ErrorCategory::NoUpstream;
    }

    let summary = stderr.lines().next().unwrap_or(stderr).trim().to_string();
    ErrorCategory::Unknown(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_network_errors() {
        assert_eq!(
            classify_error(
                "fatal: unable to access 'https://...': Could not resolve host: github.com"
            ),
            ErrorCategory::Network,
        );
        assert_eq!(
            classify_error("fatal: unable to access 'https://...': Connection refused"),
            ErrorCategory::Network,
        );
    }

    #[test]
    fn classify_auth_errors() {
        assert_eq!(
            classify_error("fatal: Authentication failed for 'https://...'"),
            ErrorCategory::Auth,
        );
        assert_eq!(
            classify_error(
                "fatal: could not read Username for 'https://...': terminal prompts disabled"
            ),
            ErrorCategory::Auth,
        );
        assert_eq!(
            classify_error("fatal: Could not read from remote repository."),
            ErrorCategory::Auth,
        );
    }

    #[test]
    fn classify_conflict() {
        assert_eq!(
            classify_error("CONFLICT (content): Merge conflict in foo.rs\nAutomatic merge failed; fix conflicts and then commit."),
            ErrorCategory::Conflict,
        );
    }

    #[test]
    fn classify_dirty_worktree() {
        assert_eq!(
            classify_error("error: Your local changes to the following files would be overwritten by checkout:\n  foo.rs\nPlease commit your changes or stash them before you switch branches."),
            ErrorCategory::DirtyWorkTree,
        );
    }

    #[test]
    fn classify_branch_not_found() {
        assert_eq!(
            classify_error(
                "error: pathspec 'no-such-branch' did not match any file(s) known to git"
            ),
            ErrorCategory::BranchNotFound,
        );
    }

    #[test]
    fn classify_no_upstream() {
        assert_eq!(
            classify_error("There is no tracking information for the current branch."),
            ErrorCategory::NoUpstream,
        );
    }

    #[test]
    fn classify_unknown_fallback() {
        let kind = classify_error("something unexpected happened\nsecond line");
        assert!(
            matches!(kind, ErrorCategory::Unknown(msg) if msg == "something unexpected happened")
        );
    }
}
