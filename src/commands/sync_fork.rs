use std::time::Duration;

use anstream::println;
use clap::Parser;
use color_eyre::Result;
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use rand::random_range;
use tokio::{time::sleep, try_join};

use crate::{
    commands::utils::{SPINNER_TICK_RATE, environment::VHS},
    credential::handle_token,
    github::github_client::{GitHub, WINGET_PKGS, WINGET_PKGS_FULL_NAME},
    terminal::Hyperlinkable,
};

/// Merges changes from microsoft/winget-pkgs into the fork repository
#[derive(Parser)]
#[clap(visible_aliases = ["sync", "merge-upstream"])]
pub struct SyncFork {
    /// Merges changes even if the fork's default branch is not fast-forward. This is not
    /// recommended as you should instead have a clean default branch that has not diverged from the
    /// upstream default branch
    #[arg(short, long)]
    force: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl SyncFork {
    pub async fn run(self) -> Result<()> {
        if *VHS {
            return Self::vhs().await;
        }

        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        // Fetch repository data from both upstream and fork repositories asynchronously
        let (winget_pkgs, fork) = try_join!(
            github.get_winget_pkgs().send(),
            github
                .get_username()
                .and_then(|username| github.get_winget_pkgs().owner(username).send()),
        )?;

        // Check whether the fork is already up-to-date with upstream by their latest commit OID's
        if winget_pkgs.default_branch_oid == fork.default_branch_oid {
            println!(
                "{} is already {} with {}",
                fork.full_name.hyperlink(&fork.url).blue(),
                "up-to-date".green(),
                winget_pkgs.full_name.hyperlink(&winget_pkgs.url).blue()
            );
            return Ok(());
        }

        // Calculate how many commits upstream is ahead of fork
        let new_commits_count = winget_pkgs.commit_count - fork.commit_count;
        let commit_label = match new_commits_count {
            1 => "commit",
            _ => "commits",
        };

        // Show an indeterminate progress bar while upstream changes are being merged
        let pb = ProgressBar::new_spinner().with_message(format!(
            "Merging {new_commits_count} upstream {commit_label} from {} into {}",
            winget_pkgs.full_name.blue(),
            fork.full_name.blue(),
        ));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        github
            .merge_upstream(
                &fork.default_branch_ref_id,
                winget_pkgs.default_branch_oid,
                self.force,
            )
            .await?;

        pb.finish_and_clear();

        println!(
            "{} merged {new_commits_count} upstream {commit_label} from {} into {}",
            "Successfully".green(),
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
