//! Repository resolution: match user-supplied targets to registered repositories.

use std::fmt;
use std::path::Path;

use crate::repository::Repository;

/// Match a user-supplied `target` against registered repositories.
///
/// Matches by exact canonical path first, then by last path component
/// (directory name). Returns an error if the directory-name match is
/// ambiguous (multiple repos share the same name).
pub fn match_repo<'a>(
    repos: &'a [Repository],
    target: &str,
) -> std::result::Result<&'a Repository, MatchError> {
    let target_path = Path::new(target);

    // 1. Exact canonical path match.
    if let Some(repo) = repos.iter().find(|r| r.path == target_path) {
        return Ok(repo);
    }

    // 2. Last path component match.
    let matches: Vec<&Repository> = repos
        .iter()
        .filter(|r| {
            r.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name == target)
        })
        .collect();

    match matches.len() {
        0 => Err(MatchError::NotFound(target.to_string())),
        1 => Ok(matches[0]),
        _ => {
            let paths: Vec<String> = matches
                .iter()
                .map(|r| r.path.display().to_string())
                .collect();
            Err(MatchError::Ambiguous {
                name: target.to_string(),
                paths,
            })
        }
    }
}

#[derive(Debug)]
pub enum MatchError {
    NotFound(String),
    Ambiguous { name: String, paths: Vec<String> },
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "no repository matching '{name}'"),
            Self::Ambiguous { name, paths } => {
                write!(f, "'{name}' is ambiguous, matches multiple repositories:")?;
                for p in paths {
                    write!(f, "\n  {p}")?;
                }
                write!(f, "\nUse the full path to disambiguate.")
            }
        }
    }
}

impl std::error::Error for MatchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn match_repo_by_exact_path() {
        let repos = vec![
            Repository::new(PathBuf::from("/code/alpha"), Some("fp1".into())),
            Repository::new(PathBuf::from("/code/beta"), Some("fp2".into())),
        ];
        let found = match_repo(&repos, "/code/alpha").unwrap();
        assert_eq!(found.path, Path::new("/code/alpha"));
    }

    #[test]
    fn match_repo_by_dir_name() {
        let repos = vec![
            Repository::new(PathBuf::from("/code/alpha"), Some("fp1".into())),
            Repository::new(PathBuf::from("/code/beta"), Some("fp2".into())),
        ];
        let found = match_repo(&repos, "beta").unwrap();
        assert_eq!(found.path, Path::new("/code/beta"));
    }

    #[test]
    fn match_repo_ambiguous_errors() {
        let repos = vec![
            Repository::new(PathBuf::from("/a/shared"), Some("fp1".into())),
            Repository::new(PathBuf::from("/b/shared"), Some("fp2".into())),
        ];
        let err = match_repo(&repos, "shared").unwrap_err();
        assert!(matches!(err, MatchError::Ambiguous { .. }));
    }

    #[test]
    fn match_repo_not_found_errors() {
        let repos = vec![Repository::new(
            PathBuf::from("/code/alpha"),
            Some("fp1".into()),
        )];
        let err = match_repo(&repos, "nope").unwrap_err();
        assert!(matches!(err, MatchError::NotFound(_)));
    }
}
