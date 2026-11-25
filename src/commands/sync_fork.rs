use std::{fmt, time::Duration};

use anstream::println;
use clap::Parser;
use color_eyre::{Result, eyre::Context};
use indicatif::{ProgressBar, ProgressFinish};
use owo_colors::OwoColorize;
use rand::random_range;
use secrecy::SecretString;
use tokio::{time::sleep, try_join};
use tracing::warn;

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    github::{Divergence, MergeType, Repository, client::GitHub, upstream},
    terminal::Hyperlinkable,
    token::TokenManager,
};

/// Merges changes from the upstream repository (microsoft/winget-pkgs by default) into the fork
/// repository
#[derive(Parser)]
#[clap(visible_aliases = ["sync-fork", "merge-upstream"])]
pub struct SyncFork {
    /// The fork to sync, if it is not `<you>/<upstream name>`, e.g. a renamed fork or one owned
    /// by an organization
    #[arg(long, value_name = "OWNER/NAME")]
    fork: Option<Repository>,

    /// The owner of the fork, if it is not the authenticated user. Ignored when --fork is given
    #[arg(long, env = "KOMAC_FORK_OWNER", value_name = "OWNER")]
    fork_owner: Option<String>,

    /// The branch to sync. Without this, the fork's default branch is synced with upstream's
    /// default branch. With it, the branch is synced with upstream's branch of the same name
    #[arg(long)]
    branch: Option<String>,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<SecretString>,
}

impl SyncFork {
    pub async fn run(self) -> Result<()> {
        if *VHS {
            return Self::vhs().await;
        }

        // Read the global upstream once here so everything below receives it explicitly
        let upstream = upstream();

        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(token_manager)?;

        let fork = match explicit_fork(self.fork, self.fork_owner, upstream) {
            Some(fork) => fork,
            None => Repository::new(github.get_username().await?, upstream.name()),
        };

        let (upstream_info, fork_info) = try_join!(
            github.get_repository(upstream),
            github.get_repository(&fork),
        )?;

        let branches = Branches::new(
            self.branch.as_deref(),
            &fork_info.default_branch_name,
            &upstream_info.default_branch_name,
        );

        let upstream_commits = match github
            .compare_fork(&fork, branches.fork, upstream, branches.upstream)
            .await?
        {
            Divergence::Identical => {
                println!(
                    "{} is already {} with {}",
                    fork_info.full_name.hyperlink(&fork_info.url).blue(),
                    "up-to-date".green(),
                    upstream_info.full_name.hyperlink(&upstream_info.url).blue()
                );
                return Ok(());
            }
            Divergence::Ahead { by } => {
                println!(
                    "{} is not behind {}, so there is nothing to merge. It has {} that {} does not",
                    fork_info.full_name.hyperlink(&fork_info.url).blue(),
                    upstream_info.full_name.hyperlink(&upstream_info.url).blue(),
                    Commits(by),
                    upstream_info.full_name,
                );
                return Ok(());
            }
            Divergence::Behind { by } => by,
            Divergence::Diverged { behind, ahead } => {
                warn!(
                    "{} has diverged from {}: it has {} that {} does not. Syncing creates a merge commit, and fails if the changes conflict",
                    fork_info.full_name,
                    upstream_info.full_name,
                    Commits(ahead),
                    upstream_info.full_name,
                );
                behind
            }
        };

        let commits = Commits(upstream_commits);

        // Show an indeterminate progress bar while upstream changes are being merged. It is
        // cleared when dropped, so a failed sync does not leave it on screen above the error.
        let pb = ProgressBar::new_spinner()
            .with_finish(ProgressFinish::AndClear)
            .with_message(format!(
                "Merging {}",
                MergeSummary {
                    commits,
                    upstream: upstream_info.full_name.blue(),
                    fork: fork_info.full_name.blue(),
                }
            ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        let response = github
            .sync_fork(&fork, branches.fork)
            .await
            .with_context(|| {
                format!(
                    "Failed to merge {}",
                    MergeSummary {
                        commits,
                        upstream: &upstream_info.full_name,
                        fork: &fork_info.full_name,
                    }
                )
            })?;

        pb.finish_and_clear();

        match response.merge_type {
            MergeType::Merge | MergeType::FastForward => println!(
                "{} merged {} ({})",
                "Successfully".green(),
                MergeSummary {
                    commits,
                    upstream: upstream_info.full_name.hyperlink(&upstream_info.url).blue(),
                    fork: fork_info.full_name.hyperlink(&fork_info.url).blue(),
                },
                response.merge_type,
            ),
            // GitHub found nothing to merge even though the comparison said otherwise, e.g.
            // because the fork was synced in between. Its message says why.
            MergeType::None => println!(
                "{}: {}",
                fork_info.full_name.hyperlink(&fork_info.url).blue(),
                response.message
            ),
        }

        Ok(())
    }

    async fn vhs() -> Result<()> {
        let fork = format!("octocat/{}", upstream().name());
        let summary = MergeSummary {
            commits: Commits(random_range(50..=500)),
            upstream: upstream().blue(),
            fork: fork.blue(),
        };

        let pb = ProgressBar::new_spinner()
            .with_finish(ProgressFinish::AndClear)
            .with_message(format!("Merging {summary}"));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        sleep(Duration::from_secs(1)).await;

        pb.finish_and_clear();

        println!("{} merged {summary}", "Successfully".green());

        Ok(())
    }
}

/// Returns the fork named on the command line: `--fork` as given, or `--fork-owner` with
/// upstream's name. `None` means the fork belongs to the authenticated user.
fn explicit_fork(
    fork: Option<Repository>,
    fork_owner: Option<String>,
    upstream: &Repository,
) -> Option<Repository> {
    fork.or_else(|| Some(Repository::new(fork_owner?, upstream.name().to_owned())))
}

/// The branch to sync in the fork, and the upstream branch it is compared with.
#[derive(Debug, PartialEq, Eq)]
struct Branches<'a> {
    fork: &'a str,
    upstream: &'a str,
}

impl<'a> Branches<'a> {
    /// Uses each repository's default branch, unless a branch was given.
    ///
    /// A given branch is compared with upstream's branch of the same name, matching
    /// `gh repo sync --branch`. GitHub's API docs don't say which upstream branch merge-upstream
    /// uses for a non-default branch.
    fn new(branch: Option<&'a str>, fork_default: &'a str, upstream_default: &'a str) -> Self {
        match branch {
            Some(branch) => Self {
                fork: branch,
                upstream: branch,
            },
            None => Self {
                fork: fork_default,
                upstream: upstream_default,
            },
        }
    }
}

/// A commit count that displays with the correctly pluralized noun, e.g. `1 commit` or
/// `12 commits`, without allocating.
#[derive(Copy, Clone)]
struct Commits(u32);

impl fmt::Display for Commits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let noun = if self.0 == 1 { "commit" } else { "commits" };
        write!(f, "{} {noun}", self.0)
    }
}

/// Describes a sync, e.g. `5 commits from microsoft/winget-pkgs into octocat/winget-pkgs`.
///
/// The repositories are generic so each message can style them (color, hyperlinks) as it needs.
struct MergeSummary<Upstream, Fork> {
    commits: Commits,
    upstream: Upstream,
    fork: Fork,
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
    use clap::Parser;
    use rstest::rstest;

    use super::{Branches, Commits, MergeSummary, SyncFork, explicit_fork};
    use crate::github::Repository;

    fn upstream() -> Repository {
        Repository::new("microsoft", "winget-pkgs")
    }

    #[test]
    fn explicit_fork_prefers_fork_over_owner() {
        assert_eq!(
            explicit_fork(
                Some(Repository::new("octocat", "renamed-pkgs")),
                Some("contoso".to_owned()),
                &upstream(),
            ),
            Some(Repository::new("octocat", "renamed-pkgs"))
        );
    }

    #[test]
    fn explicit_fork_from_owner_uses_upstream_name() {
        assert_eq!(
            explicit_fork(None, Some("contoso".to_owned()), &upstream()),
            Some(Repository::new("contoso", "winget-pkgs"))
        );
    }

    #[test]
    fn explicit_fork_is_none_without_flags() {
        assert_eq!(explicit_fork(None, None, &upstream()), None);
    }

    #[test]
    fn branches_default_to_each_default_branch() {
        assert_eq!(
            Branches::new(None, "main", "master"),
            Branches {
                fork: "main",
                upstream: "master"
            }
        );
    }

    #[test]
    fn given_branch_is_used_on_both_sides() {
        assert_eq!(
            Branches::new(Some("release"), "main", "master"),
            Branches {
                fork: "release",
                upstream: "release"
            }
        );
    }

    #[test]
    fn parses_fork_and_branch() {
        let args =
            SyncFork::try_parse_from(["sync", "--fork", "octocat/renamed-pkgs", "--branch", "dev"])
                .unwrap();

        assert_eq!(args.fork, Some(Repository::new("octocat", "renamed-pkgs")));
        assert_eq!(args.branch.as_deref(), Some("dev"));
    }

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
