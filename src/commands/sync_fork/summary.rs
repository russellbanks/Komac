use std::fmt;

/// A commit count that displays with the correctly pluralized noun, e.g. `1 commit` or
/// `12 commits`, without allocating.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Commits(pub u32);

impl fmt::Display for Commits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let noun = if self.0 == 1 { "commit" } else { "commits" };
        write!(f, "{} {noun}", self.0)
    }
}

/// Describes a sync, e.g. `5 commits from microsoft/winget-pkgs into octocat/winget-pkgs`.
///
/// The repositories are generic so each message can style them (color, hyperlinks) as it needs.
pub struct MergeSummary<Upstream, Fork> {
    pub commits: Commits,
    pub upstream: Upstream,
    pub fork: Fork,
}

impl<Upstream: fmt::Display, Fork: fmt::Display> fmt::Display for MergeSummary<Upstream, Fork> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} from {} into {}",
            self.commits, self.upstream, self.fork
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Commits, MergeSummary};

    #[rstest]
    #[case(1, "1 commit from microsoft/winget-pkgs into octocat/winget-pkgs")]
    #[case(5, "5 commits from microsoft/winget-pkgs into octocat/winget-pkgs")]
    fn merge_summary(#[case] count: u32, #[case] expected: &str) {
        let summary = MergeSummary {
            commits: Commits(count),
            upstream: "microsoft/winget-pkgs",
            fork: "octocat/winget-pkgs",
        };

        assert_eq!(summary.to_string(), expected);
    }

    #[rstest]
    #[case(0, "0 commits")]
    #[case(1, "1 commit")]
    #[case(2, "2 commits")]
    #[case(347, "347 commits")]
    fn commit_count(#[case] count: u32, #[case] expected: &str) {
        assert_eq!(Commits(count).to_string(), expected);
    }
}
