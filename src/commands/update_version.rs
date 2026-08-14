use std::{
    collections::BTreeSet,
    num::{NonZeroU32, NonZeroUsize},
    path::PathBuf,
};

use anstream::println;
use clap::Parser;
use color_eyre::eyre::{Error, Result, bail};
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use secrecy::SecretString;
use tokio::try_join;
use winget_types::{
    PackageIdentifier, PackageVersion,
    url::{DecodedUrl, ReleaseNotesUrl},
};

use crate::{
    analysis::Analyzer,
    commands::utils::{SPINNER_TICK_RATE, SubmitOption},
    download::Downloader,
    github::{
        GITHUB_HOST, GitHubError, WINGET_PKGS_FULL_NAME,
        client::{GitHub, GitHubValues},
        utils::{PackagePath, pull_request::Change},
    },
    manifests::{Url, print_changes},
    token::TokenManager,
};

/// Add a version to a pre-existing package
#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
pub struct UpdateVersion {
    /// The package's unique identifier
    #[arg(value_name = "PACKAGE_IDENTIFIER")]
    identifier: PackageIdentifier,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    version: PackageVersion,

    /// The list of package installers
    #[arg(short, long, num_args = 1.., required = true, value_hint = clap::ValueHint::Url)]
    urls: Vec<Url>,

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_downloads: NonZeroUsize,

    /// List of issues that updating this package would resolve
    #[arg(long)]
    resolves: Vec<NonZeroU32>,

    /// Automatically submit a pull request
    #[arg(short, long)]
    submit: bool,

    /// URL to package's release notes
    #[arg(long, value_hint = clap::ValueHint::Url)]
    release_notes_url: Option<ReleaseNotesUrl>,

    /// Name of external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH")]
    created_with: Option<String>,

    /// URL to external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH_URL", value_hint = clap::ValueHint::Url)]
    created_with_url: Option<DecodedUrl>,

    /// Directory to output the manifests to
    #[arg(short, long, env = "OUTPUT_DIRECTORY", value_hint = clap::ValueHint::DirPath)]
    output: Option<PathBuf>,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Run without submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// Package version to replace
    #[arg(short, long, num_args = 0..=1, default_missing_value = "latest")]
    replace: Option<PackageVersion>,

    /// Skip checking for existing pull requests
    #[arg(long, env)]
    skip_pr_check: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<SecretString>,
}

impl UpdateVersion {
    pub async fn run(mut self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token.take()).await?;
        let github = GitHub::new(&token_manager)?;

        let mut package = github
            .get_versioned_package(&self.identifier, &self.version)
            .await?;

        println!(
            "Latest version of {}: {}",
            self.identifier,
            package.latest_version()
        );

        if !self.skip_pr_check && !self.dry_run && !package.prompt_existing_pr()? {
            return Ok(());
        }

        let replace_version = self
            .resolve_replace_version(package.versions(), package.latest_version())?
            .cloned();

        let downloader = Downloader::new_with_concurrent(self.concurrent_downloads)?;
        let (mut github_values, mut files) = try_join!(
            self.fetch_github_values(&github).map_err(Error::new),
            downloader.download(self.urls.iter().cloned()),
        )?;

        let manifests = package.manifests_mut().unwrap();

        let download_results = files.analyze().await?;

        manifests.installer.package_version = self.version.clone();
        manifests.installer.installers = download_results
            .into_values()
            .flat_map(Analyzer::into_installers)
            .collect();
        manifests.installer.optimize();

        manifests.update(
            &self.version,
            &mut github_values,
            self.release_notes_url.as_ref(),
        );

        let mut changes = manifests.create(
            &self.identifier,
            &self.version,
            self.created_with.as_deref(),
        );

        if self.dry_run {
            print_changes(changes.iter().map(Change::manifest));
            return Ok(());
        }

        let submit_option =
            SubmitOption::prompt(&mut changes, &self.identifier, &self.version, self.submit)?;

        let package_path = PackagePath::new(&self.identifier, Some(&self.version), None);
        if let Some(output) = self
            .output
            .as_ref()
            .map(|out| out.join(package_path.as_str()))
        {
            changes.write_to(output.as_path()).await?;
            println!(
                "{} written all manifest files to {}",
                "Successfully".green(),
                output.display()
            );
        }

        if submit_option.is_exit() {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {} {}",
            self.identifier, self.version
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request = github
            .add_version()
            .identifier(&self.identifier)
            .version(&self.version)
            .versions(package.versions())
            .changes(changes)
            .maybe_replace_version(replace_version.as_ref())
            .issue_resolves(&self.resolves)
            .maybe_created_with(self.created_with.as_deref())
            .maybe_created_with_url(self.created_with_url.as_ref())
            .send()
            .await?;

        pr_progress.finish_and_clear();

        pull_request.print_success();

        if self.open_pr {
            open::that(pull_request.url().as_str())?;
        }

        Ok(())
    }

    fn resolve_replace_version<'a>(
        &'a self,
        versions: &'a BTreeSet<PackageVersion>,
        latest_version: &'a PackageVersion,
    ) -> Result<Option<&'a PackageVersion>> {
        let replace_version = self
            .replace
            .as_ref()
            .map(|version| {
                if version.is_latest() {
                    latest_version
                } else {
                    version
                }
            })
            .filter(|&version| version.as_str() != self.version.as_str());

        if let Some(version) = replace_version
            && !versions.contains(version)
            && let Some(closest) = version.closest(versions)
        {
            bail!(
                "Replacement version {version} does not exist in {WINGET_PKGS_FULL_NAME}. The closest version is {closest}"
            )
        }

        Ok(replace_version)
    }

    async fn fetch_github_values(
        &self,
        github: &GitHub,
    ) -> Result<Option<GitHubValues>, GitHubError> {
        if let Some(url) = self
            .urls
            .iter()
            .find(|url| url.host_str() == Some(GITHUB_HOST))
        {
            github
                .get_all_values_from_url(url.clone().into_inner())
                .await
                .transpose()
        } else {
            Ok(None)
        }
    }
}
