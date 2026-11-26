use std::{borrow::Cow, fmt, str::FromStr, sync::OnceLock};

use thiserror::Error;

use super::{GITHUB_HOST, MICROSOFT, WINGET_PKGS};

static UPSTREAM: OnceLock<Repository> = OnceLock::new();

/// Sets the upstream repository that manifests are read from and pull requests are opened
/// against.
///
/// Only the first call has an effect.
pub fn set_upstream(repository: Repository) {
    let _ = UPSTREAM.set(repository);
}

/// Returns the upstream repository that manifests are read from and pull requests are opened
/// against, defaulting to `microsoft/winget-pkgs`.
pub fn upstream() -> &'static Repository {
    UPSTREAM.get_or_init(Repository::default)
}

/// A GitHub repository identified by its owner and name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    owner: Cow<'static, str>,
    name: Cow<'static, str>,
}

impl Repository {
    pub const WINGET_PKGS: Self = Self {
        owner: Cow::Borrowed(MICROSOFT),
        name: Cow::Borrowed(WINGET_PKGS),
    };

    /// Creates a repository from its owner and name.
    #[must_use]
    pub fn new<O, N>(owner: O, name: N) -> Self
    where
        O: Into<Cow<'static, str>>,
        N: Into<Cow<'static, str>>,
    {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    #[must_use]
    #[inline]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns `true` if `full_name` (in `owner/name` form) refers to this repository.
    ///
    /// GitHub owner and repository names are case-insensitive.
    #[must_use]
    pub fn matches_full_name(&self, full_name: &str) -> bool {
        full_name.split_once('/').is_some_and(|(owner, name)| {
            owner.eq_ignore_ascii_case(&self.owner) && name.eq_ignore_ascii_case(&self.name)
        })
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::WINGET_PKGS
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected a GitHub repository in the form `owner/name`, but found `{0}`")]
pub struct RepositoryParseError(String);

impl FromStr for Repository {
    type Err = RepositoryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = || RepositoryParseError(s.to_owned());

        let trimmed = s.trim();

        // Accept a full GitHub URL, e.g. `https://github.com/owner/name`
        let path = ["https://", "http://", ""]
            .into_iter()
            .find_map(|scheme| {
                trimmed
                    .strip_prefix(scheme)?
                    .strip_prefix(GITHUB_HOST)?
                    .strip_prefix('/')
            })
            .unwrap_or(trimmed);

        let path = path.trim_end_matches('/');
        let path = path.strip_suffix(".git").unwrap_or(path);

        let (owner, name) = path.split_once('/').ok_or_else(error)?;

        let is_valid = |part: &str| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_' | '.'))
        };

        if !is_valid(owner) || !is_valid(name) {
            return Err(error());
        }

        Ok(Self {
            owner: Cow::Owned(owner.to_owned()),
            name: Cow::Owned(name.to_owned()),
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Repository;

    #[test]
    fn new_from_owner_and_name() {
        let repository = Repository::new("octocat", "winget-pkgs");

        assert_eq!(repository.owner(), "octocat");
        assert_eq!(repository.name(), "winget-pkgs");
        assert_eq!(repository.to_string(), "octocat/winget-pkgs");
    }

    #[test]
    fn default_is_winget_pkgs() {
        assert_eq!(Repository::default().to_string(), "microsoft/winget-pkgs");
    }

    #[rstest]
    #[case("microsoft/winget-pkgs")]
    #[case("contoso/private-pkgs")]
    #[case("https://github.com/contoso/private-pkgs")]
    #[case("https://github.com/contoso/private-pkgs/")]
    #[case("https://github.com/contoso/private-pkgs.git")]
    #[case("github.com/contoso/private-pkgs")]
    #[case(" contoso/private-pkgs ")]
    fn parse_valid(#[case] input: &str) {
        let repository = input.parse::<Repository>().unwrap();
        assert!(!repository.owner().is_empty());
        assert!(!repository.name().is_empty());
        assert_eq!(
            repository.to_string().parse::<Repository>().unwrap(),
            repository
        );
    }

    #[test]
    fn parse_url_extracts_owner_and_name() {
        let repository = "https://github.com/contoso/private-pkgs"
            .parse::<Repository>()
            .unwrap();
        assert_eq!(repository.owner(), "contoso");
        assert_eq!(repository.name(), "private-pkgs");
    }

    #[rstest]
    #[case("")]
    #[case("owner")]
    #[case("owner/")]
    #[case("/name")]
    #[case("a/b/c")]
    #[case("https://gitlab.com/owner/name")]
    #[case("owner/na me")]
    fn parse_invalid(#[case] input: &str) {
        assert!(input.parse::<Repository>().is_err());
    }

    #[test]
    fn matches_full_name_is_case_insensitive() {
        let repository = Repository::default();
        assert!(repository.matches_full_name("microsoft/winget-pkgs"));
        assert!(repository.matches_full_name("Microsoft/Winget-Pkgs"));
        assert!(!repository.matches_full_name("microsoft/winget-cli"));
        assert!(!repository.matches_full_name("winget-pkgs"));
    }
}
