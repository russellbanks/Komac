use std::num::NonZeroU32;

use crate::commands::utils::SPINNER_TICK_RATE;
use crate::credential::handle_token;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::prompts::prompt::handle_inquire_error;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use anstream::println;
use clap::Parser;
use color_eyre::eyre::{bail, Result};
use indicatif::ProgressBar;
use inquire::validator::{MaxLengthValidator, MinLengthValidator};
use inquire::{Confirm, Text};
use owo_colors::OwoColorize;

/// Remove a version from winget-pkgs
///
/// To remove a package, all versions of that package must be removed
#[derive(Parser)]
pub struct RemoveVersion {
    /// The package's unique identifier
    #[arg()]
    package_identifier: PackageIdentifier,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    package_version: PackageVersion,

    #[arg(short = 'r', long = "reason")]
    deletion_reason: Option<String>,

    /// List of issues that removing this version would resolve
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

    #[arg(short, long)]
    submit: bool,

    /// Don't show the package removal warning
    #[arg(long)]
    no_warning: bool,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl RemoveVersion {
    const MIN_REASON_LENGTH: usize = 4;
    const MAX_REASON_LENGTH: usize = 1000;

    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        if !self.no_warning {
            println!(
                "{}",
                "Packages should only be removed when necessary".yellow()
            );
        }
        let github = GitHub::new(&token)?;
        let versions = github.get_versions(&self.package_identifier).await?;

        if !versions.contains(&self.package_version) {
            bail!(
                "{} version {} does not exist in {WINGET_PKGS_FULL_NAME}",
                self.package_identifier,
                self.package_version,
            );
        }

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!(
            "Latest version of {}: {latest_version}",
            &self.package_identifier
        );
        let deletion_reason = match self.deletion_reason {
            Some(reason) => reason,
            None => Text::new(&format!(
                "Give a reason for removing {} version {}",
                &self.package_identifier, &self.package_version
            ))
            .with_validator(MinLengthValidator::new(Self::MIN_REASON_LENGTH))
            .with_validator(MaxLengthValidator::new(Self::MAX_REASON_LENGTH))
            .prompt()
            .map_err(handle_inquire_error)?,
        };
        let should_remove_manifest = self.submit
            || Confirm::new(&format!(
                "Would you like to make a pull request to remove {} {}?",
                self.package_identifier, self.package_version
            ))
            .prompt()
            .map_err(handle_inquire_error)?;

        if !should_remove_manifest {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request to remove {} version {}",
            self.package_identifier, self.package_version
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs().send().await?;
        let fork = github.get_winget_pkgs().owner(&current_user).send().await?;

        let pull_request_url = github
            .remove_version()
            .identifier(&self.package_identifier)
            .version(&self.package_version)
            .reason(deletion_reason)
            .fork_owner(&current_user)
            .fork(&fork)
            .winget_pkgs(&winget_pkgs)
            .maybe_issue_resolves(self.resolves)
            .send()
            .await?;

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}
