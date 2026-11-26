use crate::github::graphql::github_schema as schema;

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

    /// Returns the state's lowercase name, which translations use to choose their wording.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Merged => "merged",
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}
