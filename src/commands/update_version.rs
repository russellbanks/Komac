use std::{
    collections::BTreeSet,
    io::{Read, Seek},
    mem,
    num::{NonZeroU32, NonZeroUsize},
    sync::Arc,
};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{Result, bail};
use indicatif::ProgressBar;
use itertools::Itertools;
use owo_colors::OwoColorize;
use strsim::levenshtein;
use winget_types::{
    PackageIdentifier, PackageVersion,
    installer::{InstallerType, MinimumOSVersion, NestedInstallerFiles},
    url::{DecodedUrl, ReleaseNotesUrl},
};

use crate::{
    commands::utils::{
        SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, write_changes_to_dir,
    },
    credential::handle_token,
    download::{Download, Downloader},
    download_file::process_files,
    github::{
        github_client::{GITHUB_HOST, GitHub, WINGET_PKGS_FULL_NAME},
        utils::{PackagePath, pull_request::pr_changes},
    },
    installers::zip::Zip,
    manifests::Url,
    match_installers::match_installers,
    terminal::Hyperlinkable,
    traits::{LocaleExt, path::NormalizePath},
};

/// Add a version to a pre-existing package
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

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_downloads: NonZeroUsize,

    /// List of issues that updating this package would resolve
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

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
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        let package_identifier = Arc::new(self.package_identifier);
        let package_version = Arc::new(self.package_version);

        let existing_pr = tokio::spawn({
            let github = github.clone();
            let package_identifier = Arc::clone(&package_identifier);
            let package_version = Arc::clone(&package_version);
            async move {
                github
                    .get_existing_pull_request(&package_identifier, &package_version)
                    .await
            }
        });

        let versions = github.get_versions(&package_identifier).await?;

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!("Latest version of {package_identifier}: {latest_version}",);

        let replace_version = self.replace.as_ref().map(|version| {
            if version.is_latest() {
                latest_version
            } else {
                version
            }
        });

        if let Some(version) = replace_version
            && !versions.contains(version)
            && let Some(closest) = version.closest(&versions)
        {
            bail!(
                "Replacement version {version} does not exist in {WINGET_PKGS_FULL_NAME}. The closest version is {closest}"
            )
        }

        if let Some(pull_request) = existing_pr.await??
            && !self.skip_pr_check
            && !self.dry_run
            && !prompt_existing_pull_request(&package_identifier, &package_version, &pull_request)?
        {
            return Ok(());
        }

        let manifests = tokio::spawn({
            let github = github.clone();
            let package_identifier = Arc::clone(&package_identifier);
            let latest_version = latest_version.clone();
            async move {
                github
                    .get_manifests(&package_identifier, &latest_version)
                    .await
            }
        });

        let github_values = tokio::spawn({
            let github = github.clone();
            let github_url = self
                .urls
                .iter()
                .find(|url| url.host_str() == Some(GITHUB_HOST))
                .cloned();
            async move {
                github_url
                    .map(|url| github.get_all_values_from_url(url.into_inner()))
                    .unwrap_or_default()
                    .await
            }
        });

        let downloader = Downloader::new_with_concurrent(self.concurrent_downloads);
        let mut files = downloader
            .download(
                &self
                    .urls
                    .into_iter()
                    .unique()
                    .map(Download::new)
                    .collect::<Vec<_>>(),
            )
            .await?;
        let mut download_results = process_files(&mut files).await?;
        let installer_results = download_results
            .iter_mut()
            .flat_map(|(_url, analyser)| mem::take(&mut analyser.installers))
            .collect::<Vec<_>>();
        let mut manifests = manifests.await??;
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
        manifests.default_locale.package_version = (*package_version).clone();
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
                let mut installer = new_installer.clone().merge_with(previous_installer);
                installer.r#type = installer_type;
                installer.url.clone_from(&new_installer.url);
                installer.nested_installer_files = fix_relative_paths(
                    if installer.nested_installer_files.is_empty() {
                        manifests.installer.nested_installer_files.clone()
                    } else {
                        installer.nested_installer_files
                    },
                    analyser.zip.as_ref(),
                );
                for entry in &mut installer.apps_and_features_entries {
                    entry.deduplicate(&manifests.default_locale);
                }
                installer
            })
            .collect::<Vec<_>>();

        manifests.installer.package_version = (*package_version).clone();
        manifests.installer.minimum_os_version = manifests
            .installer
            .minimum_os_version
            .filter(|minimum_os_version| *minimum_os_version != MinimumOSVersion::new(10, 0, 0, 0));
        manifests.installer.installers = installers;
        manifests.installer.optimize();

        let mut github_values = match github_values.await? {
            Some(future) => Some(future?),
            None => None,
        };

        manifests.default_locale.update(
            &package_version,
            &mut github_values,
            self.release_notes_url.as_ref(),
        );

        manifests.locales.iter_mut().for_each(|locale| {
            locale.update(
                &package_version,
                &mut github_values,
                self.release_notes_url.as_ref(),
            );
        });

        manifests.version.update(&package_version);

        let package_path = PackagePath::new(&package_identifier, Some(&package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self.output.map(|out| out.join(package_path.as_str())) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        let submit_option = SubmitOption::prompt(
            &mut changes,
            &package_identifier,
            &package_version,
            self.submit,
            self.dry_run,
        )?;

        if submit_option == SubmitOption::Exit {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {package_identifier} {package_version}",
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request_url = github
            .add_version()
            .identifier(&package_identifier)
            .version(&package_version)
            .versions(&versions)
            .changes(changes)
            .maybe_replace_version(replace_version)
            .maybe_issue_resolves(self.resolves)
            .maybe_created_with(self.created_with)
            .maybe_created_with_url(self.created_with_url)
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
}

fn fix_relative_paths<R: Read + Seek>(
    nested_installer_files: BTreeSet<NestedInstallerFiles>,
    zip: Option<&Zip<R>>,
) -> BTreeSet<NestedInstallerFiles> {
    nested_installer_files
        .into_iter()
        .filter_map(|nested_installer_files| {
            if let Some(zip) = zip {
                return if zip
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
                };
            }
            None
        })
        .collect::<BTreeSet<_>>()
}
