mod divergence;
mod status;

pub use divergence::Divergence;
use reqwest::Method;
use serde::Deserialize;
use status::Status;

use super::transport::SendJson;
use crate::github::{GitHubError, client::GitHub, upstream};

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
    /// Compares the default branch of the fork owned by `fork_owner` with upstream's default
    /// branch.
    pub async fn compare_upstream(&self, fork_owner: &str) -> Result<Divergence, GitHubError> {
        let path = format!(
            "repos/{fork_owner}/{name}/compare/HEAD...{owner}:HEAD",
            name = upstream().name(),
            owner = upstream().owner(),
        );

        self.rest(Method::GET, &path)
            .send_json::<CompareCommits>()
            .await
            .map(CompareCommits::fork_divergence)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{CompareCommits, Divergence};

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
