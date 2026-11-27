use anstream::println;
use bitflags::bitflags;
use clap::Parser;
use color_eyre::Result;
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use inquire::MultiSelect;
use secrecy::SecretString;

use crate::{
    commands::utils::SPINNER_TICK_RATE, github::client::GitHub, i18n::fl,
    prompts::handle_inquire_error, terminal::success, token::TokenManager,
};

#[derive(Parser)]
#[command(about = fl!("command-cleanup"))]
#[clap(visible_alias = "clean")]
pub struct Cleanup {
    #[arg(long, help = fl!("arg-cleanup-only-merged"))]
    only_merged: bool,

    #[arg(long, help = fl!("arg-cleanup-only-closed"))]
    only_closed: bool,

    #[arg(short, long, env = "CI", help = fl!("arg-cleanup-all"))]
    all: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

impl Cleanup {
    pub async fn run(self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(token_manager)?;

        let merge_state = MergeState::from((self.only_merged, self.only_closed));

        let pb = ProgressBar::new_spinner()
            .with_message(fl!("cleanup-retrieving", state = merge_state.id()));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        // Get all fork branches with an associated pull request to the upstream repository
        let (pr_branch_map, repository_id) = github
            .get_username()
            .and_then(|username| github.get_branches(username, merge_state))
            .await?;

        pb.finish_and_clear();

        // Exit if there are no branches to delete
        if pr_branch_map.is_empty() {
            println!("{}", fl!("cleanup-no-branches", state = merge_state.id()));
            return Ok(());
        }

        let chosen_pr_branches = if self.all {
            pr_branch_map.keys().collect()
        } else {
            // Show a multi-selection prompt for which branches to delete, with all options pre-selected
            MultiSelect::new(
                &fl!("prompt-select-branches"),
                pr_branch_map.keys().collect(),
            )
            .with_all_selected_by_default()
            .with_page_size(10)
            .prompt()
            .map_err(handle_inquire_error)?
        };

        if chosen_pr_branches.is_empty() {
            println!("{}", fl!("cleanup-none-deleted"));
            return Ok(());
        }

        // Get branch names from chosen pull requests
        let branches_to_delete = chosen_pr_branches
            .into_iter()
            .filter_map(|pull_request| pr_branch_map.get(pull_request).map(String::as_str))
            .collect::<Vec<_>>();

        pb.reset();
        pb.set_message(fl!("cleanup-deleting", count = branches_to_delete.len()));
        pb.enable_steady_tick(SPINNER_TICK_RATE);

        let branches_count = branches_to_delete.len();

        github
            .delete_branches(&repository_id, branches_to_delete)
            .await?;

        pb.finish_and_clear();

        println!(
            "{}",
            fl!(
                "cleanup-deleted",
                success = success(),
                count = branches_count
            )
        );

        Ok(())
    }
}

// Using bitflags instead of an enum to allow combining multiple states (MERGED, CLOSED)
bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct MergeState: u8 {
        const MERGED = 1 << 0;
        const CLOSED = 1 << 1;
    }
}

impl MergeState {
    /// Returns the state's name, which translations use to choose their wording.
    const fn id(self) -> &'static str {
        match self {
            Self::MERGED => "merged",
            Self::CLOSED => "closed",
            _ => "any",
        }
    }
}

impl From<(bool, bool)> for MergeState {
    fn from((only_merged, only_closed): (bool, bool)) -> Self {
        match (only_merged, only_closed) {
            (true, false) => Self::MERGED,
            (false, true) => Self::CLOSED,
            _ => Self::all(),
        }
    }
}
