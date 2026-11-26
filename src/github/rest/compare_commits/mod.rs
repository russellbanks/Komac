mod status;

use reqwest::Method;
use serde::Deserialize;
pub use status::CompareStatus;

use super::transport::SendJson;
use crate::github::{GitHubError, Repository, client::GitHub};

/// The subset of GitHub's "Compare two commits" response that is needed.
///
/// `status`, `ahead_by` and `behind_by` describe the head commit relative to the base commit.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Comparison {
    pub status: CompareStatus,
    pub ahead_by: u32,
    pub behind_by: u32,
}

impl GitHub {
    /// Compares `fork_branch` in `fork` (the base) with `upstream_branch` in `upstream` (the
    /// head). The result describes upstream's branch relative to the fork's.
    pub async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Comparison, GitHubError> {
        self.rest(
            Method::GET,
            &compare_path(fork, fork_branch, upstream, upstream_branch),
        )
        .send_json::<Comparison>()
        .await
    }
}

/// The REST path comparing the fork's branch (base) with upstream's branch (head), which is
/// qualified with upstream's owner because it lives in another repository of the same network.
fn compare_path(
    fork: &Repository,
    fork_branch: &str,
    upstream: &Repository,
    upstream_branch: &str,
) -> String {
    format!(
        "repos/{fork}/compare/{fork_branch}...{owner}:{upstream_branch}",
        owner = upstream.owner()
    )
}

#[cfg(test)]
mod tests {
    use super::compare_path;
    use crate::github::Repository;

    #[test]
    fn compare_path_uses_both_branches() {
        assert_eq!(
            compare_path(
                &Repository::new("octocat", "winget-pkgs"),
                "main",
                &Repository::new("microsoft", "winget-pkgs"),
                "master",
            ),
            "repos/octocat/winget-pkgs/compare/main...microsoft:master"
        );
    }
}
