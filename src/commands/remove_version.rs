use crate::credential::handle_token;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::graphql::create_commit::FileDeletion;
use crate::github::utils::{
    get_branch_name, get_commit_title, get_package_path, get_pull_request_body,
};
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::update_state::UpdateState;
use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use crossterm::style::Stylize;
use indicatif::ProgressBar;
use inquire::validator::{MaxLengthValidator, MinLengthValidator};
use inquire::{Confirm, Text};
use std::num::NonZeroU32;
use std::time::Duration;

#[derive(Parser)]
pub struct RemoveVersion {
    /// The package's unique identifier
    #[arg(short = 'i', long = "identifier")]
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
        println!(
            "{}",
            "Packages should only be removed when necessary".yellow()
        );
        println!();
        let github = GitHub::new(&token)?;
        let versions = github
            .get_versions(&get_package_path(&self.package_identifier, None))
            .await
            .wrap_err_with(|| {
                format!(
                    "{} does not exist in {WINGET_PKGS_FULL_NAME}",
                    self.package_identifier
                )
            })?;

        if !versions.contains(&self.package_version) {
            bail!(
                "{} version {} does not exist in {WINGET_PKGS_FULL_NAME}",
                self.package_identifier,
                self.package_version,
            );
        }

        let latest_version = versions.iter().max().unwrap();
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
            .prompt()?,
        };
        let should_remove_manifest = if self.submit {
            true
        } else {
            Confirm::new(&format!(
                "Would you like to make a pull request to remove {} {}?",
                self.package_identifier, self.package_version
            ))
            .prompt()?
        };
        if !should_remove_manifest {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request to remove {} version {}",
            self.package_identifier, self.package_version
        ));
        pr_progress.enable_steady_tick(Duration::from_millis(50));

        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs(None).await?;
        let fork = github.get_winget_pkgs(Some(&current_user)).await?;
        let branch_name = get_branch_name(&self.package_identifier, &self.package_version);
        let pull_request_branch = github
            .create_branch(&fork.id, &branch_name, &winget_pkgs.default_branch_oid.0)
            .await?;
        let commit_title = get_commit_title(
            &self.package_identifier,
            &self.package_version,
            &UpdateState::RemoveVersion,
        );
        let directory_content = github
            .get_directory_content(
                &current_user,
                &branch_name,
                &get_package_path(&self.package_identifier, Some(&self.package_version)),
            )
            .await?
            .collect::<Vec<_>>();
        let deletions = directory_content
            .iter()
            .map(|path| FileDeletion { path })
            .collect::<Vec<_>>();
        let _commit_url = github
            .create_commit(
                &pull_request_branch.id,
                &pull_request_branch
                    .target
                    .map(|object| object.oid.0)
                    .unwrap(),
                &commit_title,
                None,
                Some(deletions),
            )
            .await?;
        let pull_request_url = github
            .create_pull_request(
                &winget_pkgs.id,
                &fork.id,
                &format!("{current_user}:{}", pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &get_pull_request_body(self.resolves, Some(deletion_reason), None, None),
            )
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a pull request to remove {} version {}",
            "Successfully".green(),
            self.package_identifier,
            self.package_version
        );
        println!("{}", pull_request_url.as_str());

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}
