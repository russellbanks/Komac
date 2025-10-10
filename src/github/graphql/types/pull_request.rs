use std::fmt::{Display, Formatter};

use super::super::github_schema as schema;

/// The possible states of a pull request.
///
/// See <https://docs.github.com/graphql/reference/enums#pullrequeststate>.
#[derive(cynic::Enum, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PullRequestState {
    /// A pull request that has been closed without being merged.
    Closed,

    /// A pull request that has been closed by being merged.
    Merged,

    /// A pull request that is still open.
    Open,
}

impl PullRequestState {
    /// Returns `true` if the pull request has been closed without being merged.
    #[inline]
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Returns `true` if the pull request has been closed by being merged.
    #[inline]
    pub const fn is_merged(self) -> bool {
        matches!(self, Self::Merged)
    }

    /// Returns `true` if the pull request is still open.
    #[inline]
    pub const fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Merged => "a merged",
            Self::Open => "an open",
            Self::Closed => "a closed",
        }
    }
}

impl Display for PullRequestState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
