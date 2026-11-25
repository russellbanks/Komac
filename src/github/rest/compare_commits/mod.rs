mod divergence;
mod status;

pub use divergence::Divergence;
use reqwest::Method;
use serde::Deserialize;
use status::Status;

use super::transport::SendJson;
use crate::github::{GitHubError, Repository, client::GitHub};

/// The subset of GitHub's "Compare two commits" response that is needed.
///
/// `status`, `ahead_by` and `behind_by` describe the head commit relative to the base commit.
#[derive(Copy, Clone, Deserialize)]
struct CompareCommits {
    status: Status,
    ahead_by: u32,
    behind_by: u32,
}

impl CompareCommits {
    /// Interprets a comparison whose base is the fork and whose head is upstream, flipping
    /// GitHub's head-relative status so that it describes the fork.
    #[must_use]
    const fn fork_divergence(self) -> Divergence {
        match self.status {
            Status::Identical => Divergence::Identical,
            Status::Ahead => Divergence::Behind { by: self.ahead_by },
            Status::Behind => Divergence::Ahead { by: self.behind_by },
            Status::Diverged => Divergence::Diverged {
                behind: self.ahead_by,
                ahead: self.behind_by,
            },
        }
    }
}

impl GitHub {
    /// Compares `fork_branch` in `fork` with `upstream_branch` in `upstream`, describing how the
    /// fork's branch relates to upstream's.
    pub async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Divergence, GitHubError> {
        self.rest(
            Method::GET,
            &compare_path(fork, fork_branch, upstream, upstream_branch),
        )
        .send_json::<CompareCommits>()
        .await
        .map(CompareCommits::fork_divergence)
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
    use rstest::rstest;

    use super::{CompareCommits, Divergence, compare_path};
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

    /// The comparison is `fork...upstream`, so GitHub's `ahead`/`behind` describe upstream
    /// relative to the fork and must be flipped to describe the fork.
    #[rstest]
    #[case("identical", 0, 0, Divergence::Identical)]
    #[case("ahead", 12, 0, Divergence::Behind { by: 12 })]
    #[case("behind", 0, 3, Divergence::Ahead { by: 3 })]
    #[case("diverged", 12, 3, Divergence::Diverged { behind: 12, ahead: 3 })]
    fn fork_divergence(
        #[case] status: &str,
        #[case] ahead_by: u32,
        #[case] behind_by: u32,
        #[case] expected: Divergence,
    ) {
        let comparison = serde_json::from_str::<CompareCommits>(&format!(
            r#"{{"status":"{status}","ahead_by":{ahead_by},"behind_by":{behind_by},"total_commits":{ahead_by}}}"#
        ))
        .unwrap();

        assert_eq!(comparison.fork_divergence(), expected);
    }
}
