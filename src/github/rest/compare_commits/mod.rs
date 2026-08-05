mod status;

use reqwest::header::ACCEPT;
use serde::Deserialize;
pub use status::Status;

use super::{
    GITHUB_JSON_MIME, REST_API_URL, REST_API_VERSION, X_GITHUB_API_VERSION, error::RestError,
};
use crate::github::{GitHubError, MICROSOFT, WINGET_PKGS, client::GitHub};

#[derive(Copy, Clone, Deserialize)]
pub struct CompareCommits {
    status: Status,
    ahead_by: u32,
}

impl CompareCommits {
    /// Returns `true` if the commit has diverged from the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_diverged(self) -> bool {
        self.status.is_diverged()
    }

    /// Returns `true` if the commit is ahead of the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_ahead(self) -> bool {
        self.status.is_ahead()
    }

    /// Returns `true` if the commit is behind the comparison commit.
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn is_behind(self) -> bool {
        self.status.is_behind()
    }

    /// Returns `true` if the commit is identical to the comparison commit.
    #[must_use]
    #[inline]
    pub const fn is_identical(self) -> bool {
        self.status.is_identical()
    }

    /// Returns the number of commits the commit is ahead of the comparison
    /// commit.
    #[must_use]
    #[inline]
    pub const fn ahead_by(self) -> u32 {
        self.ahead_by
    }
}

impl GitHub {
    pub async fn compare_upstream(&self, fork_owner: &str) -> Result<CompareCommits, GitHubError> {
        let endpoint = format!(
            "{REST_API_URL}/repos/{fork_owner}/{WINGET_PKGS}/compare/HEAD...{MICROSOFT}:HEAD"
        );

        let response = self
            .0
            .get(endpoint)
            .header(ACCEPT, GITHUB_JSON_MIME)
            .header(X_GITHUB_API_VERSION, REST_API_VERSION)
            .send()
            .await?;

        if response.status().is_success() {
            response
                .json::<CompareCommits>()
                .await
                .map_err(GitHubError::from)
        } else {
            Err(response
                .json::<RestError>()
                .await
                .map_err(GitHubError::from)?
                .into())
        }
    }
}
