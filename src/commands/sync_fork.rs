use std::time::Duration;

use anstream::println;
use clap::Parser;
use color_eyre::{Result, eyre::Context};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use rand::random_range;
use secrecy::SecretString;
use tokio::{time::sleep, try_join};

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    github::{WINGET_PKGS, WINGET_PKGS_FULL_NAME, client::GitHub},
    terminal::Hyperlinkable,
    token::TokenManager,
};

/// Merges changes from microsoft/winget-pkgs into the fork repository
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

        let comparison = github.compare_upstream(&username).await?;
        if comparison.is_identical() {
            println!(
                "{} is already {} with {}",
                fork.full_name.hyperlink(&fork.url).blue(),
                "up-to-date".green(),
                winget_pkgs.full_name.hyperlink(&winget_pkgs.url).blue()
            );
            return Ok(());
        }

        let commit_label = match comparison.ahead_by {
            1 => "commit",
            _ => "commits",
        };

        // Show an indeterminate progress bar while upstream changes are being merged
        let pb = ProgressBar::new_spinner().with_message(format!(
            "Merging {} upstream {commit_label} from {} into {}",
            comparison.ahead_by,
            winget_pkgs.full_name.blue(),
            fork.full_name.blue(),
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        let sync_type = github
            .sync_fork(&username, &fork.default_branch_name)
            .await
            .with_context(|| {
                format!(
                    "while merging {} upstream {commit_label} from {} into {}",
                    comparison.ahead_by, winget_pkgs.full_name, fork.full_name
                )
            })?
            .merge_type;

        pb.finish_and_clear();

        println!(
            "{} merged {} upstream {commit_label} from {} into {} ({sync_type})",
            "Successfully".green(),
            comparison.ahead_by,
            winget_pkgs.full_name.hyperlink(winget_pkgs.url).blue(),
            fork.full_name.hyperlink(fork.url).blue()
        );

        Ok(())
    }

    async fn vhs() -> Result<()> {
        let merge_message = format!(
            "{} upstream commits from {} into {}",
            random_range(50..=500),
            WINGET_PKGS_FULL_NAME.blue(),
            format_args!("octocat/{WINGET_PKGS}").blue()
        );

        let pb = ProgressBar::new_spinner().with_message(format!("Merging {merge_message}"));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        sleep(Duration::from_secs(1)).await;

        pb.finish_and_clear();

        print!("{} merged {merge_message}", "Successfully".green());

        Ok(())
    }
}
