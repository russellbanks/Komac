//! The "Sync fork" API, i.e. the `merge-upstream` endpoint.
//!
//! See <https://docs.github.com/en/rest/branches/branches#sync-a-fork-branch-with-the-upstream-repository>.

mod merge_type;

pub use merge_type::MergeType;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use super::{
    super::{GitHubError, WINGET_PKGS, client::GitHub},
    GITHUB_JSON_MIME, REST_API_URL, REST_API_VERSION, X_GITHUB_API_VERSION,
    error::RestError,
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
    #[allow(unused)]
    pub message: String,
    /// The strategy GitHub used to sync the branch.
    pub merge_type: MergeType,
    /// The upstream branch GitHub synced from, including its owner.
    #[allow(unused)]
    pub base_branch: String,
}

impl GitHub {
    /// Syncs a fork branch using the `merge-upstream` endpoint. This matches
    /// GitHub's "Sync fork" behavior in the web UI.
    ///
    /// See <https://docs.github.com/rest/branches/branches#sync-a-fork-branch-with-the-upstream-repository>.
    pub async fn sync_fork(&self, fork_owner: &str, branch: &str) -> Result<Response, GitHubError> {
        let endpoint = format!("{REST_API_URL}/repos/{fork_owner}/{WINGET_PKGS}/merge-upstream");

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
                .json::<RestError>()
                .await
                .map_err(GitHubError::from)?
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

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
        const RESPONSE: &str = indoc! {r#"
            {
              "message": "Successfully fetched and fast-forwarded from upstream defunkt:main",
              "merge_type": "fast-forward",
              "base_branch": "defunkt:main"
            }
        "#};

        let response = serde_json::from_str::<Response>(RESPONSE).unwrap();

        assert_eq!(
            response.message,
            "Successfully fetched and fast-forwarded from upstream defunkt:main"
        );
        assert_eq!(response.merge_type, MergeType::FastForward);
        assert_eq!(response.base_branch, "defunkt:main");
    }

    #[test]
    fn error_response_deserializes() {
        let error = serde_json::from_str::<RestError>(
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
        let error = serde_json::from_str::<RestError>(
            r#"{
                "message": "Validation Failed"
            }"#,
        )
        .unwrap();

        assert!(error.status.is_none());
        assert_eq!(error.message, "Validation Failed");
    }
}
