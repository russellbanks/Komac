use std::{fmt, time::Duration};

use anstream::println;
use clap::Parser;
use color_eyre::{Result, eyre::Context};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use rand::random_range;
use secrecy::SecretString;
use tokio::{time::sleep, try_join};
use tracing::warn;

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    github::{Divergence, MergeType, client::GitHub, upstream},
    terminal::Hyperlinkable,
    token::TokenManager,
};

/// Merges changes from the upstream repository (microsoft/winget-pkgs by default) into the fork
/// repository
#[derive(Parser)]
#[clap(visible_aliases = ["sync-fork", "merge-upstream"])]
pub struct SyncFork {
    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<SecretString>,
}

impl SyncFork {
    pub async fn run(self) -> Result<()> {
        if *VHS {
            return Self::vhs().await;
        }

        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(token_manager)?;

        let username = github.get_username().await?;

        // Fetch repository data from both upstream and fork repositories asynchronously
        let (winget_pkgs, fork) = try_join!(
            github.get_winget_pkgs().send(),
            github.get_winget_pkgs().owner(&username).send(),
        )?;

        let upstream_commits = match github.compare_upstream(&username).await? {
            Divergence::Identical => {
                println!(
                    "{} is already {} with {}",
                    fork.full_name.hyperlink(&fork.url).blue(),
                    "up-to-date".green(),
                    winget_pkgs.full_name.hyperlink(&winget_pkgs.url).blue()
                );
                return Ok(());
            }
            Divergence::Ahead { by } => {
                println!(
                    "{} is not behind {}, so there is nothing to merge. It has {} that {} does not",
                    fork.full_name.hyperlink(&fork.url).blue(),
                    winget_pkgs.full_name.hyperlink(&winget_pkgs.url).blue(),
                    Commits(by),
                    winget_pkgs.full_name,
                );
                return Ok(());
            }
            Divergence::Behind { by } => by,
            Divergence::Diverged { behind, ahead } => {
                warn!(
                    "{} has diverged from {}: it has {} that {} does not. Syncing creates a merge commit, and fails if the changes conflict",
                    fork.full_name,
                    winget_pkgs.full_name,
                    Commits(ahead),
                    winget_pkgs.full_name,
                );
                behind
            }
        };

        // Show an indeterminate progress bar while upstream changes are being merged
        let pb = ProgressBar::new_spinner().with_message(format!(
            "Merging {} from {} into {}",
            Commits(upstream_commits),
            winget_pkgs.full_name.blue(),
            fork.full_name.blue(),
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        let response = github
            .sync_fork(&username, &fork.default_branch_name)
            .await
            .with_context(|| {
                format!(
                    "while merging {} from {} into {}",
                    Commits(upstream_commits),
                    winget_pkgs.full_name,
                    fork.full_name
                )
            })?;

        pb.finish_and_clear();

        match response.merge_type {
            MergeType::Merge | MergeType::FastForward => println!(
                "{} merged {} from {} into {} ({})",
                "Successfully".green(),
                Commits(upstream_commits),
                winget_pkgs.full_name.hyperlink(winget_pkgs.url).blue(),
                fork.full_name.hyperlink(fork.url).blue(),
                response.merge_type,
            ),
            // GitHub found nothing to merge even though the comparison said otherwise, e.g.
            // because the fork was synced in between. Its message says why.
            MergeType::None => println!(
                "{}: {}",
                fork.full_name.hyperlink(fork.url).blue(),
                response.message
            ),
        }

        Ok(())
    }

    async fn vhs() -> Result<()> {
        let merge_message = format!(
            "{} upstream commits from {} into {}",
            random_range(50..=500),
            upstream().blue(),
            format_args!("octocat/{}", upstream().name()).blue()
        );

        let pb = ProgressBar::new_spinner().with_message(format!("Merging {merge_message}"));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        sleep(Duration::from_secs(1)).await;

        pb.finish_and_clear();

        print!("{} merged {merge_message}", "Successfully".green());

        Ok(())
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Commits;

    #[rstest]
    #[case(0, "0 commits")]
    #[case(1, "1 commit")]
    #[case(2, "2 commits")]
    #[case(347, "347 commits")]
    fn commit_count(#[case] count: u32, #[case] expected: &str) {
        assert_eq!(Commits(count).to_string(), expected);
    }
}
