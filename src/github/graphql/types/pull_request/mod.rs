mod state;

use chrono::{DateTime, Utc};
pub use state::PullRequestState;
use url::Url;

use crate::github::graphql::github_schema as schema;

#[derive(cynic::QueryFragment)]
pub struct PullRequest {
    pub title: String,
    pub url: Url,
    pub state: PullRequestState,
    pub created_at: DateTime<Utc>,
}

impl PullRequest {
    /// Returns `true` if the pull request has been closed without being merged.
    #[expect(unused)]
    #[inline]
    pub const fn is_closed(&self) -> bool {
        self.state.is_closed()
    }

    /// Returns `true` if the pull request has been closed by being merged.
    #[expect(unused)]
    #[inline]
    pub const fn is_merged(&self) -> bool {
        self.state.is_merged()
    }

    /// Returns `true` if the pull request is still open.
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.state.is_open()
    }
}
