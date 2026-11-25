mod demo;
mod service;
mod summary;

use anstream::println;
use clap::Parser;
use color_eyre::Result;
use demo::DemoApi;
use indicatif::{ProgressBar, ProgressFinish};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use service::{Progress, SyncOutcome, SyncReport, SyncRequest, sync};
use summary::MergeSummary;

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    github::{Repository, client::GitHub, upstream},
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
        // Read the global upstream once here so everything below receives it explicitly
        let upstream = upstream();

        let request = SyncRequest {
            upstream,
            fork: explicit_fork(self.fork, self.fork_owner, upstream),
            branch: self.branch.as_deref(),
        };

        // VHS recordings run the same flow against a pretend GitHub, without a token
        let report = if *VHS {
            sync(&DemoApi, &request, &Spinner).await?
        } else {
            let github = GitHub::new(TokenManager::handle(self.token).await?)?;
            sync(&github, &request, &Spinner).await?
        };

        render(&report);

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

/// Shows a spinner while a merge is in progress.
struct Spinner;

impl Progress for Spinner {
    fn merging(&self, summary: &MergeSummary<&Repository, &Repository>) -> impl Sized {
        // Cleared when dropped, so a failed merge does not leave it on screen above the error
        let pb = ProgressBar::new_spinner()
            .with_finish(ProgressFinish::AndClear)
            .with_message(format!(
                "Merging {}",
                MergeSummary {
                    commits: summary.commits,
                    upstream: summary.upstream.blue(),
                    fork: summary.fork.blue(),
                }
            ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);
        pb
    }
}

/// Prints the outcome of a sync.
fn render(report: &SyncReport) {
    let SyncReport {
        upstream,
        fork,
        outcome,
    } = report;

    let upstream_link = upstream.repository.hyperlink(&upstream.url);
    let fork_link = fork.repository.hyperlink(&fork.url);

    match outcome {
        SyncOutcome::UpToDate => println!(
            "{} is already {} with {}",
            fork_link.blue(),
            "up-to-date".green(),
            upstream_link.blue()
        ),
        SyncOutcome::ForkAhead(commits) => println!(
            "{} is not behind {}, so there is nothing to merge. It has {commits} that {} does not",
            fork_link.blue(),
            upstream_link.blue(),
            upstream.repository,
        ),
        SyncOutcome::Merged {
            commits,
            merge_type,
        } => println!(
            "{} merged {} ({merge_type})",
            "Successfully".green(),
            MergeSummary {
                commits: *commits,
                upstream: upstream_link.blue(),
                fork: fork_link.blue(),
            },
        ),
        SyncOutcome::NothingMerged { message } => println!("{}: {message}", fork_link.blue()),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{SyncFork, explicit_fork};
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
    fn parses_fork_and_branch() {
        let args =
            SyncFork::try_parse_from(["sync", "--fork", "octocat/renamed-pkgs", "--branch", "dev"])
                .unwrap();

        assert_eq!(args.fork, Some(Repository::new("octocat", "renamed-pkgs")));
        assert_eq!(args.branch.as_deref(), Some("dev"));
    }
}
