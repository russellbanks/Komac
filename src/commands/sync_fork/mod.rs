//! `komac sync`, laid out as ports and adapters:
//!
//! - `domain`: the core's types and rules, with no knowledge of GitHub or the terminal
//! - `service`: the use case and the ports it drives ([`ForkSyncApi`](service::ForkSyncApi),
//!   [`SyncObserver`])
//! - `github_api` and `demo`: driven adapters for the real GitHub and the VHS demo
//! - this module: the driving adapter (command line), plus the terminal observer and output

mod demo;
mod domain;
mod github_api;
mod service;
mod summary;

use anstream::println;
use clap::Parser;
use color_eyre::{Report, Result, Section};
use demo::DemoApi;
use domain::{SyncError, SyncOutcome, SyncReport};
use indicatif::{ProgressBar, ProgressFinish};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use service::{SyncObserver, SyncRequest, sync};
use summary::{Commits, MergeSummary};
use tracing::warn;

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    github::{Repository, client::GitHub, upstream},
    i18n::fl,
    terminal::{Hyperlinkable, success},
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

        // VHS recordings run the same flow against a pretend host, without a token
        let result = if *VHS {
            sync(&DemoApi, &request, &TerminalObserver).await
        } else {
            let github = GitHub::new(TokenManager::handle(self.token).await?)?;
            sync(&github, &request, &TerminalObserver).await
        };

        render(&result.map_err(into_report)?);

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

/// Turns a sync error into a report, with a suggestion that names the command-line options that
/// could fix it.
fn into_report(error: SyncError) -> Report {
    let hint = cli_hint(&error);
    let report = Report::new(error);

    match hint {
        Some(hint) => report.suggestion(hint),
        None => report,
    }
}

/// A suggestion that names the command-line options that could fix `error`, if there are any.
fn cli_hint(error: &SyncError) -> Option<String> {
    match error {
        SyncError::ForkNotFound { upstream, .. } => Some(format!(
            "Fork {upstream} on GitHub first, or name your fork with --fork"
        )),
        _ => None,
    }
}

/// Shows a sync in the terminal: a warning for a diverged fork, and a spinner while merging.
struct TerminalObserver;

impl SyncObserver for TerminalObserver {
    fn diverged(&self, fork: &Repository, upstream: &Repository, ahead: Commits) {
        warn!(
            "{fork} has diverged from {upstream}: it has {ahead} that {upstream} does not. Syncing creates a merge commit, and fails if the changes conflict"
        );
    }

    fn merging(&self, summary: &MergeSummary<&Repository, &Repository>) -> impl Sized {
        // Cleared when dropped, so a failed merge does not leave it on screen above the error
        let pb = ProgressBar::new_spinner()
            .with_finish(ProgressFinish::AndClear)
            .with_message(fl!(
                "sync-merging",
                summary = MergeSummary {
                    commits: summary.commits,
                    upstream: summary.upstream.blue(),
                    fork: summary.fork.blue(),
                }
                .to_string()
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

    let message = match outcome {
        SyncOutcome::UpToDate => fl!(
            "sync-up-to-date",
            fork = fork_link.blue().to_string(),
            status = fl!("sync-up-to-date", "status").green().to_string(),
            upstream = upstream_link.blue().to_string()
        ),
        SyncOutcome::ForkAhead(commits) => fl!(
            "sync-fork-ahead",
            fork = fork_link.blue().to_string(),
            upstream = upstream_link.blue().to_string(),
            commits = commits.to_string(),
            upstream_name = upstream.repository.to_string()
        ),
        SyncOutcome::Merged { commits, kind } => fl!(
            "sync-merged",
            success = success(),
            summary = MergeSummary {
                commits: *commits,
                upstream: upstream_link.blue(),
                fork: fork_link.blue(),
            }
            .to_string(),
            kind = kind.to_string()
        ),
        SyncOutcome::NothingMerged { message } => format!("{}: {message}", fork_link.blue()),
    };
    println!("{message}");
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{SyncError, SyncFork, cli_hint, explicit_fork};
    use crate::github::Repository;

    fn upstream() -> Repository {
        Repository::new("microsoft", "winget-pkgs")
    }

    #[test]
    fn missing_fork_hint_names_fork_option() {
        let error = SyncError::ForkNotFound {
            fork: Repository::new("octocat", "winget-pkgs"),
            upstream: upstream(),
        };

        assert_eq!(
            cli_hint(&error).as_deref(),
            Some("Fork microsoft/winget-pkgs on GitHub first, or name your fork with --fork")
        );
    }

    #[test]
    fn other_errors_have_no_hint() {
        let error = SyncError::UpstreamNotFound(upstream());

        assert_eq!(cli_hint(&error), None);
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
