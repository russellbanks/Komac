use std::num::NonZeroU32;

use anstream::println;
use clap::Parser;
use color_eyre::eyre::{Result, bail};
use futures_util::TryFutureExt;
use inquire::{
    Text,
    validator::{MaxLengthValidator, MinLengthValidator},
};
use owo_colors::OwoColorize;
use tokio::try_join;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    credential::handle_token,
    github::github_client::{GitHub, WINGET_PKGS_FULL_NAME},
    prompts::{handle_inquire_error, text::confirm_prompt},
};

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
        let token = handle_token(self.token.as_deref()).await?;
        if !self.no_warning {
            println!(
                "{}",
                "Packages should only be removed when necessary".yellow()
            );
        }
        let github = GitHub::new(&token)?;

        let (fork, winget_pkgs, versions) = try_join!(
            github
                .get_username()
                .and_then(|current_user| github.get_winget_pkgs().owner(current_user).send()),
            github.get_winget_pkgs().send(),
            github.get_versions(&self.package_identifier)
        )?;

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
            || confirm_prompt(&format!(
                "Would you like to make a pull request to remove {} {}?",
                self.package_identifier, self.package_version
            ))?;

        if !should_remove_manifest {
            return Ok(());
        }

        let pull_request_url = github
            .remove_version()
            .identifier(&self.package_identifier)
            .version(&self.package_version)
            .reason(deletion_reason)
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
