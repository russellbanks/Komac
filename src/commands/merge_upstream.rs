use std::time::Duration;

use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::hyperlink::Hyperlink;
use anstream::println;
use clap::Parser;
use color_eyre::Result;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;

/// Merges changes from microsoft/winget-pkgs into the fork repository
#[derive(Parser)]
#[clap(alias = "merge-upstream")]
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
        let token = handle_token(self.token).await?;
        let github = GitHub::new(&token)?;

        // Fetch repository data from both upstream and fork repositories asynchronously
        let winget_pkgs = github.get_winget_pkgs(None);
        let winget_pkgs_fork = github
            .get_winget_pkgs(Some(&github.get_username().await?))
            .await?;
        let winget_pkgs = winget_pkgs.await?;

        // Create hyperlinks to the repository's URLs when their full name is printed
        let winget_pkgs_hyperlink = winget_pkgs.full_name.hyperlink(winget_pkgs.url);
        let winget_pkgs_fork_hyperlink = winget_pkgs_fork.full_name.hyperlink(winget_pkgs_fork.url);

        // Check whether the fork is already up-to-date with upstream by their latest commit OID's
        if winget_pkgs.default_branch_oid == winget_pkgs_fork.default_branch_oid {
            println!(
                "{} is already {} with {}",
                winget_pkgs_fork_hyperlink.blue(),
                "up-to-date".green(),
                winget_pkgs_hyperlink.blue()
            );
            return Ok(());
        }

        // Calculate how many commits upstream is ahead of fork
        let new_commits_count = winget_pkgs.commit_count - winget_pkgs_fork.commit_count;
        let commit_label = match new_commits_count {
            1 => "commit",
            _ => "commits",
        };

        // Show an indeterminate progress bar while upstream changes are being merged
        let pb = ProgressBar::new_spinner().with_message(format!(
            "Merging {new_commits_count} upstream {commit_label} from {} into {}",
            winget_pkgs.full_name.as_str().blue(),
            winget_pkgs_fork.full_name.as_str().blue(),
        ));
        pb.enable_steady_tick(Duration::from_millis(50));

        github
            .merge_upstream(
                &winget_pkgs_fork.default_branch_ref_id,
                winget_pkgs.default_branch_oid,
                self.force,
            )
            .await?;

        pb.finish_and_clear();
        println!(
            "{} merged {new_commits_count} upstream {commit_label} from {} into {}",
            "Successfully".green(),
            winget_pkgs_hyperlink.blue(),
            winget_pkgs_fork_hyperlink.blue()
        );

        Ok(())
    }
}
