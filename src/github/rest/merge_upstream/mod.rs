//! The "Sync fork" API, i.e. the `merge-upstream` endpoint.
//!
//! See <https://docs.github.com/en/rest/branches/branches#sync-a-fork-branch-with-the-upstream-repository>.

mod merge_type;

pub use merge_type::MergeType;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{
    super::{GitHubError, client::GitHub, upstream},
    transport::SendJson,
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
        let path = format!(
            "repos/{fork_owner}/{name}/merge-upstream",
            name = upstream().name()
        );

        self.rest(Method::POST, &path)
            .json(&Body { branch })
            .send_json::<Response>()
            .await
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
}
