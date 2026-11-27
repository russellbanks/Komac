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
use secrecy::SecretString;
use tokio::try_join;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    github::{client::GitHub, upstream},
    i18n::fl,
    prompts::{handle_inquire_error, text::confirm_prompt},
    token::TokenManager,
};

#[derive(Parser)]
#[command(about = fl!("command-remove"), long_about = fl!("command-remove", "long"))]
pub struct RemoveVersion {
    #[arg(value_name = "PACKAGE_IDENTIFIER", help = fl!("arg-package-identifier"))]
    package_identifier: PackageIdentifier,

    #[arg(short = 'v', long = "version", help = fl!("arg-package-version"))]
    package_version: PackageVersion,

    #[arg(short = 'r', long = "reason")]
    deletion_reason: Option<String>,

    #[arg(long, help = fl!("arg-remove-resolves"))]
    resolves: Vec<NonZeroU32>,

    #[arg(short, long)]
    submit: bool,

    #[arg(long, help = fl!("arg-remove-no-warning"))]
    no_warning: bool,

    #[arg(long, env = "OPEN_PR", help = fl!("arg-open-pr"))]
    open_pr: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

impl RemoveVersion {
    const MIN_REASON_LENGTH: usize = 4;
    const MAX_REASON_LENGTH: usize = 1000;

    pub async fn run(self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token).await?;

        if !self.no_warning {
            println!("{}", fl!("remove-version-warning").yellow());
        }

        let github = GitHub::new(&token_manager)?;

        let (fork, winget_pkgs, versions) = try_join!(
            github
                .get_username()
                .and_then(|current_user| github.get_winget_pkgs().owner(current_user).send()),
            github.get_winget_pkgs().send(),
            github.get_versions(&self.package_identifier)
        )?;

        if !versions.contains(&self.package_version) {
            bail!(
                "{}",
                fl!(
                    "error-version-not-found",
                    identifier = self.package_identifier.as_str(),
                    version = self.package_version.to_string(),
                    upstream = upstream().to_string()
                )
            );
        }

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!(
            "{}",
            fl!(
                "latest-version",
                identifier = self.package_identifier.as_str(),
                version = latest_version.to_string()
            )
        );
        let deletion_reason = match self.deletion_reason {
            Some(reason) => reason,
            None => Text::new(&fl!(
                "prompt-removal-reason",
                identifier = self.package_identifier.as_str(),
                version = self.package_version.to_string()
            ))
            .with_validator(MinLengthValidator::new(Self::MIN_REASON_LENGTH))
            .with_validator(MaxLengthValidator::new(Self::MAX_REASON_LENGTH))
            .prompt()
            .map_err(handle_inquire_error)?,
        };
        let should_remove_manifest = self.submit
            || confirm_prompt(&fl!(
                "prompt-remove-version",
                identifier = self.package_identifier.as_str(),
                version = self.package_version.to_string()
            ))?;

        if !should_remove_manifest {
            return Ok(());
        }

        let pull_request = github
            .remove_version()
            .identifier(&self.package_identifier)
            .version(&self.package_version)
            .reason(&deletion_reason)
            .fork(&fork)
            .winget_pkgs(&winget_pkgs)
            .issue_resolves(&self.resolves)
            .send()
            .await?;

        if self.open_pr {
            open::that(pull_request.url().as_str())?;
        }

        Ok(())
    }
}
