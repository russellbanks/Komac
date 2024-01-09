use crate::credential::handle_token;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::utils::{
    get_branch_name, get_commit_title, get_full_package_path, get_package_path,
};
use crate::graphql::create_commit::FileDeletion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::update_state::UpdateState;
use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use crossterm::style::Stylize;
use inquire::validator::{MaxLengthValidator, MinLengthValidator};
use inquire::{Confirm, Text};

#[derive(Parser)]
pub struct RemoveVersion {
    #[arg(short = 'i', long = "identifier")]
    package_identifier: PackageIdentifier,

    #[arg(short = 'v', long = "version")]
    package_version: PackageVersion,

    #[arg(short = 'r', long = "reason")]
    deletion_reason: Option<String>,

    #[arg(short, long)]
    submit: bool,

    /// GitHub personal access token with the public_repo and read_org scope
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
        let github = GitHub::new(token)?;
        let versions = github
            .get_versions(&get_package_path(&self.package_identifier))
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
        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs().await?;
        let fork_id = github.get_winget_pkgs_fork_id(&current_user).await?;
        let branch_name = get_branch_name(&self.package_identifier, &self.package_version);
        let pull_request_branch = github
            .create_branch(&fork_id, &branch_name, &winget_pkgs.default_branch_oid)
            .await?;
        let commit_title = get_commit_title(
            &self.package_identifier,
            &self.package_version,
            &UpdateState::RemoveVersion,
        );
        let deletions = github
            .get_directory_content(
                &current_user,
                &branch_name,
                &get_full_package_path(&self.package_identifier, &self.package_version),
            )
            .await?
            .map(|path| FileDeletion { path })
            .collect::<Vec<_>>();
        let _commit_url = github
            .create_commit(
                &pull_request_branch.id,
                &pull_request_branch.head_sha,
                &commit_title,
                None,
                Some(deletions),
            )
            .await?;
        let pull_request_url = github
            .create_pull_request(
                &winget_pkgs.id,
                &fork_id,
                &format!("{current_user}:{}", pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &format!("## {deletion_reason}"),
            )
            .await?;
        println!(
            "{} created a pull request to delete {} version {}",
            "Successfully".green(),
            self.package_identifier,
            self.package_version
        );
        println!("{}", pull_request_url.as_str());

        Ok(())
    }
}
