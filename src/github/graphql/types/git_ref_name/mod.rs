mod git_ref_kind;

use std::{fmt, str::FromStr};

pub use git_ref_kind::GitRefKind;
use git_ref_kind::GitRefKindParseError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// <https://docs.github.com/graphql/reference/scalars#gitrefname>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GitRefName {
    kind: GitRefKind,
    name: String,
}

impl GitRefName {
    /// Creates a new Git reference name of a branch.
    ///
    /// This will serialize as `refs/heads/{name}`.
    pub fn new_branch<S: Into<String>>(name: S) -> Self {
        Self {
            kind: GitRefKind::Heads,
            name: name.into(),
        }
    }

    /// Returns `true` if the Git reference kind is heads.
    ///
    /// For example, `refs/heads/main`.
    #[expect(unused)]
    #[inline]
    pub const fn is_heads(&self) -> bool {
        self.kind.is_heads()
    }

    /// Returns `true` if the Git reference kind is tags.
    ///
    /// For example, `refs/tags/v1.0.0`.
    #[expect(unused)]
    #[inline]
    pub const fn is_tags(&self) -> bool {
        self.kind.is_tags()
    }

    /// Returns `true` if the Git reference kind is remotes.
    ///
    /// For example, `refs/remotes/origin/main`.
    #[expect(unused)]
    #[inline]
    pub const fn is_remote(&self) -> bool {
        self.kind.is_remote()
    }

    /// Returns `true` if the Git reference kind is a pull request.
    ///
    /// For example, `refs/pull/42`.
    #[expect(unused)]
    #[inline]
    pub const fn is_pull(&self) -> bool {
        self.kind.is_pull()
    }

    /// Returns the Git reference kind.
    #[inline]
    pub const fn kind(&self) -> GitRefKind {
        self.kind
    }
}

impl fmt::Display for GitRefName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "refs/{}/{}", self.kind(), self.name)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GitRefNameError {
    #[error("git reference name must be in the format refs/<type>/<name> but was {0}")]
    InvalidFormat(String),
    #[error(transparent)]
    Kind(#[from] GitRefKindParseError),
}

impl FromStr for GitRefName {
    type Err = GitRefNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(3, '/');

        if split.next() != Some("refs") {
            return Err(Self::Err::InvalidFormat(s.into()));
        }

        let kind = split
            .next()
            .ok_or_else(|| Self::Err::InvalidFormat(s.into()))?
            .parse()?;

        Ok(Self {
            kind,
            name: split
                .next()
                .ok_or_else(|| Self::Err::InvalidFormat(s.into()))?
                .into(),
        })
    }
}

impl Serialize for GitRefName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GitRefName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let git_ref_name = <&str>::deserialize(deserializer)?;
        git_ref_name.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::{GitRefKind, GitRefName};

    #[rstest]
    #[case("refs/heads/main", GitRefKind::Heads, "main")]
    #[case("refs/tags/v1.0.0", GitRefKind::Tags, "v1.0.0")]
    #[case("refs/remotes/origin/main", GitRefKind::Remotes, "origin/main")]
    #[case("refs/pull/42/head", GitRefKind::Pull, "42/head")]
    fn parse_known_kinds(
        #[case] input: &str,
        #[case] expected_kind: GitRefKind,
        #[case] expected_name: &str,
    ) {
        let parsed = GitRefName::from_str(input).unwrap();
        assert_eq!(parsed.kind, expected_kind);
        assert_eq!(parsed.name, expected_name);
        assert_eq!(parsed.to_string(), input);
    }
}
