use std::{
    collections::BTreeSet,
    io::{Read, Seek},
    mem,
    num::{NonZeroU32, NonZeroUsize},
};

use anstream::println;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::eyre::{Error, Result, bail, ensure};
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use strsim::levenshtein;
use tokio::try_join;
use winget_types::{
    PackageIdentifier, PackageVersion,
    installer::{InstallerType, MinimumOSVersion, NestedInstallerFiles},
    url::{DecodedUrl, ReleaseNotesUrl},
};

use crate::{
    analysis::installers::Zip,
    commands::utils::{
        SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, write_changes_to_dir,
    },
    download::{DownloadedFile, Downloader},
    download_file::process_files,
    github::{
        GITHUB_HOST, GitHubError, WINGET_PKGS_FULL_NAME,
        client::{GitHub, GitHubValues},
        graphql::get_existing_pull_request::PullRequest,
        utils::{PackagePath, pull_request::pr_changes},
    },
    manifests::Url,
    match_installers::match_installers,
    terminal::Hyperlinkable,
    token::TokenManager,
    traits::{LocaleExt, path::NormalizePath},
};

/// Add a version to a pre-existing package
#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
pub struct UpdateVersion {
    /// The package's unique identifier
    #[arg()]
    package_identifier: PackageIdentifier,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    package_version: PackageVersion,

    /// The list of package installers
    #[arg(short, long, num_args = 1.., required = true, value_hint = clap::ValueHint::Url)]
    urls: Vec<Url>,

    /// The list of files to use instead of downloading urls
    #[arg(short, long, num_args = 1.., requires = "urls", value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    files: Vec<Utf8PathBuf>,

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
    output: Option<Utf8PathBuf>,

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
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl UpdateVersion {
    pub async fn run(self) -> Result<()> {
        if !self.files.is_empty() {
            ensure!(
                self.urls.len() == self.files.len(),
                "Number of URLs ({}) must match number of files ({})",
                self.urls.len(),
                self.files.len()
            );
        }

        let token = TokenManager::handle(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        let (versions, existing_pr) = try_join!(
            github.get_versions(&self.package_identifier),
            github.get_existing_pull_request(&self.package_identifier, &self.package_version),
        )?;

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!(
            "Latest version of {}: {latest_version}",
            self.package_identifier
        );

        let replace_version = self.resolve_replace_version(&versions, latest_version)?;

        if self.should_abort_for_existing_pr(existing_pr)? {
            return Ok(());
        }

        let (mut manifests, mut github_values, mut files) = try_join!(
            github
                .get_manifests(&self.package_identifier, latest_version)
                .map_err(Error::new),
            self.fetch_github_values(&github).map_err(Error::new),
            async {
                if self.files.is_empty() {
                    let downloader = Downloader::new_with_concurrent(self.concurrent_downloads)?;
                    downloader.download(self.urls.iter().cloned()).await
                } else {
                    self.files
                        .iter()
                        .zip(self.urls.iter().cloned())
                        .map(|(path, url)| DownloadedFile::from_local(path, url))
                        .collect::<Result<Vec<_>>>()
                }
            },
        )?;

        let mut download_results = process_files(&mut files).await?;
        let installer_results = download_results
            .iter_mut()
            .flat_map(|(_url, analyser)| mem::take(&mut analyser.installers))
            .collect::<Vec<_>>();
        let previous_installers = mem::take(&mut manifests.installer.installers)
            .into_iter()
            .map(|mut installer| {
                if manifests.installer.r#type.is_some() {
                    installer.r#type = manifests.installer.r#type;
                }
                if manifests.installer.nested_installer_type.is_some() {
                    installer.nested_installer_type = manifests.installer.nested_installer_type;
                }
                if manifests.installer.scope.is_some() {
                    installer.scope = manifests.installer.scope;
                }
                installer
            })
            .collect::<Vec<_>>();
        manifests.default_locale.package_version = self.package_version.clone();
        let matched_installers = match_installers(previous_installers, &installer_results);
        let installers = matched_installers
            .into_iter()
            .map(|(previous_installer, new_installer)| {
                let analyser = &download_results[&new_installer.url];
                let installer_type = match previous_installer.r#type {
                    Some(InstallerType::Portable) => previous_installer.r#type,
                    _ => match new_installer.r#type {
                        Some(InstallerType::Portable) => previous_installer.r#type,
                        _ => new_installer.r#type,
                    },
                };

                let previous_nested_files = previous_installer.nested_installer_files.clone();

                let mut installer = new_installer.clone().merge_with(previous_installer);
                installer.r#type = installer_type;
                installer.url.clone_from(&new_installer.url);

                let nested_files_to_fix = [
                    &previous_nested_files,
                    &manifests.installer.nested_installer_files,
                    &installer.nested_installer_files,
                ]
                .into_iter()
                .find(|files| !files.is_empty())
                .cloned();

                if let Some(nested_files) = nested_files_to_fix {
                    installer.nested_installer_files =
                        fix_relative_paths(nested_files, analyser.zip.as_ref());
                }

                for entry in &mut installer.apps_and_features_entries {
                    entry.deduplicate(&manifests.default_locale);
                }
                installer
            })
            .collect::<Vec<_>>();

        manifests.installer.package_version = self.package_version.clone();
        manifests.installer.minimum_os_version = manifests
            .installer
            .minimum_os_version
            .filter(|minimum_os_version| *minimum_os_version != MinimumOSVersion::new(10, 0, 0, 0));
        manifests.installer.installers = installers;
        manifests.installer.optimize();

        manifests.default_locale.update(
            &self.package_version,
            &mut github_values,
            self.release_notes_url.as_ref(),
        );

        manifests.locales.iter_mut().for_each(|locale| {
            locale.update(
                &self.package_version,
                &mut github_values,
                self.release_notes_url.as_ref(),
            );
        });

        manifests.version.update(&self.package_version);

        let package_path =
            PackagePath::new(&self.package_identifier, Some(&self.package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&self.package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self
            .output
            .as_ref()
            .map(|out| out.join(package_path.as_str()))
        {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        let submit_option = SubmitOption::prompt(
            &mut changes,
            &self.package_identifier,
            &self.package_version,
            self.submit,
            self.dry_run,
        )?;

        if submit_option.is_exit() {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {} {}",
            self.package_identifier, self.package_version
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request_url = github
            .add_version()
            .identifier(&self.package_identifier)
            .version(&self.package_version)
            .versions(&versions)
            .changes(changes)
            .maybe_replace_version(replace_version)
            .issue_resolves(&self.resolves)
            .maybe_created_with(self.created_with.as_deref())
            .maybe_created_with_url(self.created_with_url.as_ref())
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a {} to {WINGET_PKGS_FULL_NAME}",
            "Successfully".green(),
            "pull request".hyperlink(&pull_request_url)
        );

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
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
            .filter(|&version| version.as_str() != self.package_version.as_str());

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

    fn should_abort_for_existing_pr<T>(&self, existing_pr: T) -> Result<bool>
    where
        T: Into<Option<PullRequest>>,
    {
        if let Some(ref pull_request) = existing_pr.into()
            && !self.skip_pr_check
            && !self.dry_run
            && !prompt_existing_pull_request(
                &self.package_identifier,
                &self.package_version,
                pull_request,
            )?
        {
            return Ok(true);
        }

        Ok(false)
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

fn fix_relative_paths<R: Read + Seek>(
    nested_installer_files: BTreeSet<NestedInstallerFiles>,
    zip: Option<&Zip<R>>,
) -> BTreeSet<NestedInstallerFiles> {
    let Some(zip) = zip else {
        return nested_installer_files;
    };

    nested_installer_files
        .into_iter()
        .filter_map(|nested_installer_files| {
            if zip
                .possible_installer_files
                .contains(&nested_installer_files.relative_file_path.normalize())
            {
                Some(nested_installer_files)
            } else {
                zip.possible_installer_files
                    .iter()
                    .min_by_key(|file_path| {
                        levenshtein(
                            file_path.as_str(),
                            nested_installer_files.relative_file_path.as_str(),
                        )
                    })
                    .map(|path| NestedInstallerFiles {
                        relative_file_path: path.to_path_buf(),
                        ..nested_installer_files
                    })
            }
        })
        .collect::<BTreeSet<_>>()
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    ensure!(path.exists(), "{path} does not exist");
    ensure!(path.is_file(), "{path} is not a file");
    Ok(path.to_path_buf())
}
