use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the category of a Git reference.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitRefKind {
    /// A branch reference, e.g. `refs/heads/main`.
    Heads,

    /// A tag reference, e.g. `refs/tags/v1.0.0`.
    Tags,

    /// A remote-tracking branch, e.g. `refs/remotes/origin/main`.
    Remotes,

    /// A pull request reference, e.g. `refs/pull/42`.
    Pull,
}

impl GitRefKind {
    /// Returns `true` if the Git reference kind is heads.
    ///
    /// For example, `refs/heads/main`.
    #[inline]
    pub const fn is_heads(self) -> bool {
        matches!(self, Self::Heads)
    }

    /// Returns `true` if the Git reference kind is tags.
    ///
    /// For example, `refs/tags/v1.0.0`.
    #[inline]
    pub const fn is_tags(self) -> bool {
        matches!(self, Self::Tags)
    }

    /// Returns `true` if the Git reference kind is remotes.
    ///
    /// For example, `refs/remotes/origin/main`.
    #[inline]
    pub const fn is_remote(self) -> bool {
        matches!(self, Self::Remotes)
    }

    /// Returns `true` if the Git reference kind is a pull request.
    ///
    /// For example, `refs/pull/42`.
    #[inline]
    pub const fn is_pull(self) -> bool {
        matches!(self, Self::Pull)
    }

    /// Returns the Git reference kind as a static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Heads => "heads",
            Self::Tags => "tags",
            Self::Remotes => "remotes",
            Self::Pull => "pull",
        }
    }
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("git reference kind did not match `heads`, `tags`, `remotes`, or `pull`")]
pub struct GitRefKindParseError;

impl FromStr for GitRefKind {
    type Err = GitRefKindParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heads" => Ok(Self::Heads),
            "tags" => Ok(Self::Tags),
            "remotes" => Ok(Self::Remotes),
            "pull" => Ok(Self::Pull),
            _ => Err(GitRefKindParseError),
        }
    }
}

impl fmt::Display for GitRefKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
