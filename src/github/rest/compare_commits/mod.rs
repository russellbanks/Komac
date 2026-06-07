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
    pub status: Status,
    pub ahead_by: u32,
}

impl CompareCommits {
    /// Returns `true` if the commit is identical to the comparison commit.
    #[must_use]
    #[inline]
    pub const fn is_identical(&self) -> bool {
        self.status.is_identical()
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
