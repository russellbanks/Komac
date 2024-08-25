use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use anstream::println;
use bitflags::bitflags;
use clap::Parser;
use color_eyre::Result;
use indicatif::ProgressBar;
use inquire::MultiSelect;
use owo_colors::OwoColorize;
use std::fmt::{Display, Formatter};
use std::time::Duration;

/// Finds branches from the fork of winget-pkgs that have had a merged or closed pull request to
/// microsoft/winget-pkgs from them, prompting for which ones to delete
#[derive(Parser)]
#[clap(visible_alias = "clean")]
pub struct Cleanup {
    /// Only delete merged branches
    #[arg(long)]
    only_merged: bool,

    /// Only delete closed branches
    #[arg(long)]
    only_closed: bool,

    /// Automatically delete all relevant branches
    #[arg(short, long, env = "CI")]
    all: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Cleanup {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        let github = GitHub::new(&token)?;

        let merge_state = MergeState::from_bools(self.only_merged, self.only_closed);

        let pb = ProgressBar::new_spinner().with_message(format!(
            "Retrieving branches that have a {merge_state} pull request associated with them"
        ));
        pb.enable_steady_tick(Duration::from_millis(50));

        // Get all fork branches with an associated pull request to microsoft/winget-pkgs
        let (pr_branch_map, repository_id) = github
            .get_branches(&github.get_username().await?, &merge_state)
            .await?;

        pb.finish_and_clear();

        // Exit if there are no branches to delete
        if pr_branch_map.is_empty() {
            println!(
                "There are no {} pull requests with branches that can be deleted",
                merge_state.blue()
            );
            return Ok(());
        }

        let chosen_pr_branches = if self.all {
            pr_branch_map.keys().collect()
        } else {
            // Show a multi-selection prompt for which branches to delete, with all options pre-selected
            MultiSelect::new(
                "Please select branches to delete",
                pr_branch_map.keys().collect(),
            )
            .with_all_selected_by_default()
            .with_page_size(10)
            .prompt()?
        };

        if chosen_pr_branches.is_empty() {
            println!("No branches have been deleted");
            return Ok(());
        }

        // Get branch names from chosen pull requests
        let branches_to_delete = chosen_pr_branches
            .into_iter()
            .filter_map(|pull_request| pr_branch_map.get(pull_request).map(String::as_str))
            .collect::<Vec<_>>();

        let branch_label = match branches_to_delete.len() {
            1 => "branch",
            _ => "branches",
        };

        pb.reset();
        pb.set_message(format!(
            "Deleting {} selected {branch_label}",
            branches_to_delete.len(),
        ));
        pb.enable_steady_tick(Duration::from_millis(50));

        github
            .delete_branches(&repository_id, &branches_to_delete)
            .await?;

        pb.finish_and_clear();
        println!(
            "{} deleted {} selected {branch_label}",
            "Successfully".green(),
            branches_to_delete.len().blue(),
        );

        Ok(())
    }
}

// Using bitflags instead of an enum to allow combining multiple states (MERGED, CLOSED)
bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct MergeState: u8 {
        const MERGED = 1 << 0;
        const CLOSED = 1 << 1;
    }
}

impl Display for MergeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::MERGED => "merged",
                Self::CLOSED => "closed",
                _ => "merged or closed",
            }
        )
    }
}

impl MergeState {
    pub const fn from_bools(only_merged: bool, only_closed: bool) -> Self {
        match (only_merged, only_closed) {
            (true, false) => Self::MERGED,
            (false, true) => Self::CLOSED,
            _ => Self::all(),
        }
    }
}
