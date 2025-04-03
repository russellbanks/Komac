use std::{
    collections::BTreeSet,
    io::{Read, Seek},
    mem,
    num::{NonZeroU8, NonZeroU32},
};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{Result, bail};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use reqwest::Client;
use strsim::levenshtein;
use winget_types::{
    installer::{InstallerType, MinimumOSVersion, NestedInstallerFiles},
    shared::{
        PackageIdentifier, PackageVersion,
        url::{DecodedUrl, ReleaseNotesUrl},
    },
    traits::Closest,
};

use crate::{
    commands::utils::{
        SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, prompt_submit_option,
        write_changes_to_dir,
    },
    credential::{get_default_headers, handle_token},
    download_file::{download_urls, process_files},
    github::{
        github_client::{GITHUB_HOST, GitHub, WINGET_PKGS_FULL_NAME},
        utils::{get_package_path, pull_request::pr_changes},
    },
    installers::zip::Zip,
    match_installers::match_installers,
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
    urls: Vec<DecodedUrl>,

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

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
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let existing_pr =
            github.get_existing_pull_request(&self.package_identifier, &self.package_version);

        let versions = github.get_versions(&self.package_identifier).await?;

        let latest_version = versions.last().unwrap_or_else(|| unreachable!());
        println!(
            "Latest version of {}: {latest_version}",
            self.package_identifier
        );

        let replace_version = self.replace.as_ref().map(|version| {
            if version.is_latest() {
                latest_version
            } else {
                version
            }
        });

        if let Some(version) = replace_version {
            if !versions.contains(version) {
                let closest = version.closest(&versions).unwrap_or_else(|| unreachable!());
                bail!(
                    "Replacement version {version} does not exist in {WINGET_PKGS_FULL_NAME}. The closest version is {closest}"
                )
            }
        }

        if let Some(pull_request) = existing_pr.await? {
            if !(self.skip_pr_check || self.dry_run)
                && !prompt_existing_pull_request(
                    &self.package_identifier,
                    &self.package_version,
                    &pull_request,
                )?
            {
                return Ok(());
            }
        }

        let manifests = github.get_manifests(&self.package_identifier, latest_version);
        let github_values = self
            .urls
            .iter()
            .find(|url| url.host_str() == Some(GITHUB_HOST))
            .and_then(|url| github.get_all_values_from_url(url));

        let mut files = download_urls(&client, self.urls, self.concurrent_downloads).await?;
        let mut download_results = process_files(&mut files).await?;
        let installer_results = download_results
            .iter_mut()
            .flat_map(|(_url, analyser)| mem::take(&mut analyser.installers))
            .collect::<Vec<_>>();
        let mut manifests = manifests.await?;
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
                installer.nested_installer_files = installer
                    .nested_installer_files
                    .or_else(|| manifests.installer.nested_installer_files.clone())
                    .and_then(|nested_installer_files| {
                        validate_relative_paths(nested_installer_files, analyser.zip.as_ref())
                    });
                if let Some(entries) = installer.apps_and_features_entries.as_mut() {
                    for entry in entries {
                        entry.deduplicate(&self.package_version, &manifests.default_locale);
                    }
                }
                installer
            })
            .collect::<Vec<_>>();

        manifests.installer.minimum_os_version = manifests
            .installer
            .minimum_os_version
            .filter(|minimum_os_version| *minimum_os_version != MinimumOSVersion::removable());
        manifests.installer.installers = installers;
        manifests
            .installer
            .reorder_keys(&self.package_identifier, &self.package_version);

        let mut github_values = match github_values {
            Some(future) => Some(future.await?),
            None => None,
        };

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
            get_package_path(&self.package_identifier, Some(&self.package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&self.package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self.output.map(|out| out.join(package_path)) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        let submit_option = prompt_submit_option(
            &mut changes,
            self.submit,
            &self.package_identifier,
            &self.package_version,
            self.dry_run,
        )?;

        if submit_option == SubmitOption::Exit {
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
            .maybe_issue_resolves(self.resolves)
            .maybe_created_with(self.created_with)
            .maybe_created_with_url(self.created_with_url)
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a pull request to {WINGET_PKGS_FULL_NAME}",
            "Successfully".green()
        );
        println!("{}", pull_request_url.as_str());

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}

fn validate_relative_paths<R: Read + Seek>(
    nested_installer_files: BTreeSet<NestedInstallerFiles>,
    zip: Option<&Zip<R>>,
) -> Option<BTreeSet<NestedInstallerFiles>> {
    let relative_paths = nested_installer_files
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
        .collect::<BTreeSet<_>>();

    if relative_paths.is_empty() {
        None
    } else {
        Some(relative_paths)
    }
}
