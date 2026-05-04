//! The "Sync fork" API, i.e. the `merge-upstream` endpoint.
//!
//! See <https://docs.github.com/en/rest/branches/branches#sync-a-fork-branch-with-the-upstream-repository>.

use color_eyre::eyre::eyre;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use super::{
    super::{GitHubError, client::GitHub},
    GITHUB_JSON_MIME, REST_API_URL, REST_API_VERSION, X_GITHUB_API_VERSION,
};

#[derive(Serialize)]
struct Body<'branch> {
    /// The name of the branch in the fork that should be synced with its
    /// upstream counterpart.
    branch: &'branch str,
}

#[derive(Deserialize)]
pub struct Response {
    /// Human-readable summary of the sync result returned by GitHub.
    #[allow(dead_code)] // For completeness
    pub message: String,
    /// The strategy GitHub used to sync the branch, such as `fast-forward`.
    pub merge_type: String,
    /// The upstream branch GitHub synced from, including its owner.
    #[allow(dead_code)] // For completeness
    pub base_branch: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    status: Option<String>,
    message: String,
}

impl From<ErrorResponse> for GitHubError {
    fn from(value: ErrorResponse) -> Self {
        let ErrorResponse { status, message } = value;
        Self::Rest(match status {
            Some(status) => eyre!("Status {status}: {message}"),
            None => eyre!("{message}"),
        })
    }
}

impl GitHub {
    /// Syncs a fork branch using the `merge-upstream` endpoint. This matches
    /// GitHub's "Sync fork" behavior in the web UI.
    ///
    /// See <https://docs.github.com/en/rest/branches/branches#sync-a-fork-branch-with-the-upstream-repository>.
    pub async fn sync_fork(&self, repository: &str, branch: &str) -> Result<Response, GitHubError> {
        let endpoint = format!("{REST_API_URL}/repos/{repository}/merge-upstream");

        let response = self
            .0
            .post(endpoint)
            .header(ACCEPT, GITHUB_JSON_MIME)
            .header(X_GITHUB_API_VERSION, REST_API_VERSION)
            .json(&Body { branch })
            .send()
            .await?;

        if response.status().is_success() {
            response.json::<Response>().await.map_err(GitHubError::from)
        } else {
            Err(response
                .json::<ErrorResponse>()
                .await
                .map_err(GitHubError::from)?
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_serializes() {
        assert_eq!(
            serde_json::to_string(&Body { branch: "main" }).unwrap(),
            r#"{"branch":"main"}"#
        );
    }

    #[test]
    fn response_deserializes() {
        let response = serde_json::from_str::<Response>(
            r#"{
                "message":"Successfully fetched and fast-forwarded from upstream defunkt:main",
                "merge_type":"fast-forward",
                "base_branch":"defunkt:main"
            }"#,
        )
        .unwrap();

        assert_eq!(
            response.message,
            "Successfully fetched and fast-forwarded from upstream defunkt:main"
        );
        assert_eq!(response.merge_type, "fast-forward");
        assert_eq!(response.base_branch, "defunkt:main");
    }

    #[test]
    fn error_response_deserializes() {
        let error = serde_json::from_str::<ErrorResponse>(
            r#"{
                "status": "422",
                "message": "refusing to allow a Personal Access Token to create or update workflow `.github/workflows/foo.yaml` without `workflow` scope"
            }"#,
        ).unwrap();

        assert_eq!(error.status.as_deref(), Some("422"));
        assert_eq!(
            error.message,
            "refusing to allow a Personal Access Token to create or update workflow `.github/workflows/foo.yaml` without `workflow` scope"
        );
    }

    #[test]
    fn error_response_without_status_deserializes() {
        let error = serde_json::from_str::<ErrorResponse>(
            r#"{
                "message": "Validation Failed"
            }"#,
        )
        .unwrap();

        assert_eq!(error.status, None);
        assert_eq!(error.message, "Validation Failed");
    }
}
