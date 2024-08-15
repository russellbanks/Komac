use std::collections::BTreeSet;
use std::io::{Read, Seek};
use std::mem;
use std::num::{NonZeroU32, NonZeroU8};
use std::time::Duration;

use anstream::println;
use base64ct::Encoding;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::Result;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar};
use owo_colors::OwoColorize;
use reqwest::Client;
use strsim::levenshtein;

use crate::commands::utils::{
    prompt_existing_pull_request, prompt_submit_option, write_changes_to_dir, SubmitOption,
};
use crate::credential::{get_default_headers, handle_token};
use crate::download_file::{download_urls, process_files};
use crate::github::github_client::{GitHub, GITHUB_HOST, WINGET_PKGS_FULL_NAME};
use crate::github::graphql::create_commit::FileAddition;
use crate::github::graphql::types::Base64String;
use crate::github::pr_changes::PRChangesBuilder;
use crate::github::utils::{
    get_branch_name, get_commit_title, get_package_path, get_pull_request_body,
};
use crate::installers::zip::Zip;
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, InstallationMetadata, Installer, NestedInstallerFiles, Scope,
    UpgradeBehavior,
};
use crate::match_installers::match_installers;
use crate::types::installer_type::InstallerType;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::path::NormalizePath;
use crate::types::urls::url::DecodedUrl;
use crate::update_state::UpdateState;

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
    resolves: Vec<NonZeroU32>,

    /// Automatically submit a pull request
    #[arg(short, long)]
    submit: bool,

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

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl UpdateVersion {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        let github = GitHub::new(&token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let existing_pr =
            github.get_existing_pull_request(&self.package_identifier, &self.package_version);

        let versions = github.get_versions(&self.package_identifier).await?;

        let latest_version = versions.iter().max().unwrap();
        println!(
            "Latest version of {}: {latest_version}",
            self.package_identifier
        );

        if let Some(pull_request) = existing_pr.await? {
            if !self.dry_run
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
        let multi_progress = MultiProgress::new();
        let mut files = stream::iter(download_urls(&client, self.urls, &multi_progress))
            .buffer_unordered(self.concurrent_downloads.get() as usize)
            .try_collect::<Vec<_>>()
            .await?;
        multi_progress.clear()?;
        let github_values = files
            .iter()
            .find(|download| download.url.host_str() == Some(GITHUB_HOST))
            .map(|download| {
                let parts = download.url.path_segments().unwrap().collect::<Vec<_>>();
                github.get_all_values(
                    parts[0].to_owned(),
                    parts[1].to_owned(),
                    parts[4..parts.len() - 1].join("/"),
                )
            });
        let download_results = process_files(&mut files).await?;
        let installer_results = download_results
            .iter()
            .map(|(url, download)| Installer {
                architecture: download.architecture.unwrap_or_default(),
                installer_type: Some(download.installer_type),
                nested_installer_type: download
                    .zip
                    .as_ref()
                    .and_then(|zip| zip.nested_installer_type),
                scope: download.scope.or_else(|| Scope::get_from_url(url.as_str())),
                installer_url: url.clone(),
                ..Installer::default()
            })
            .collect::<Vec<_>>();
        let mut manifests = manifests.await?;
        let previous_installers = mem::take(&mut manifests.installer_manifest.installers)
            .into_iter()
            .map(|mut installer| {
                if manifests.installer_manifest.installer_type.is_some() {
                    installer.installer_type = manifests.installer_manifest.installer_type;
                }
                if manifests.installer_manifest.nested_installer_type.is_some() {
                    installer.nested_installer_type =
                        manifests.installer_manifest.nested_installer_type;
                }
                if manifests.installer_manifest.scope.is_some() {
                    installer.scope = manifests.installer_manifest.scope;
                }
                installer
            })
            .collect::<Vec<_>>();
        let matched_installers = match_installers(previous_installers, &installer_results);
        let installers = matched_installers
            .into_iter()
            .map(|(mut previous_installer, new_installer)| {
                let analyser = download_results.get(&new_installer.installer_url).unwrap();
                if analyser.product_language.is_some() {
                    previous_installer
                        .installer_locale
                        .clone_from(&analyser.product_language);
                }
                if analyser.platform.is_some() {
                    previous_installer.platform.clone_from(&analyser.platform);
                }
                if analyser.minimum_os_version.is_some() {
                    previous_installer
                        .minimum_os_version
                        .clone_from(&analyser.minimum_os_version);
                } else if previous_installer.minimum_os_version
                    == Some(MinimumOSVersion::removable())
                {
                    previous_installer.minimum_os_version = None;
                }
                previous_installer.installer_type = match previous_installer.installer_type {
                    Some(InstallerType::Portable) => previous_installer.installer_type,
                    _ => match new_installer.installer_type {
                        Some(InstallerType::Portable) => previous_installer.installer_type,
                        _ => new_installer.installer_type,
                    },
                };
                if let Some(nested_installer_type) = analyser
                    .zip
                    .as_ref()
                    .and_then(|zip| zip.nested_installer_type)
                {
                    previous_installer.nested_installer_type = Some(nested_installer_type);
                }
                previous_installer.nested_installer_files = previous_installer
                    .nested_installer_files
                    .or_else(|| manifests.installer_manifest.nested_installer_files.clone())
                    .or_else(|| {
                        analyser
                            .zip
                            .as_ref()
                            .and_then(|zip| zip.nested_installer_files.clone())
                    })
                    .and_then(|nested_installer_files| {
                        validate_relative_paths(nested_installer_files, &analyser.zip)
                    });
                if previous_installer.scope.is_none() {
                    previous_installer.scope = new_installer.scope;
                }
                previous_installer
                    .installer_url
                    .clone_from(&new_installer.installer_url);
                previous_installer
                    .installer_sha_256
                    .clone_from(&analyser.installer_sha_256);
                previous_installer
                    .signature_sha_256
                    .clone_from(&analyser.signature_sha_256);
                if let Some(upgrade_behaviour) = UpgradeBehavior::get(analyser.installer_type) {
                    previous_installer.upgrade_behavior = Some(upgrade_behaviour);
                }
                previous_installer.file_extensions = previous_installer
                    .file_extensions
                    .map(|mut extensions| {
                        if let Some(mut identified_extensions) = analyser.file_extensions.clone() {
                            extensions.append(&mut identified_extensions);
                        }
                        extensions
                    })
                    .or_else(|| analyser.file_extensions.clone());
                previous_installer
                    .package_family_name
                    .clone_from(&analyser.package_family_name);
                previous_installer
                    .product_code
                    .clone_from(&analyser.product_code);
                previous_installer
                    .capabilities
                    .clone_from(&analyser.capabilities);
                previous_installer
                    .restricted_capabilities
                    .clone_from(&analyser.restricted_capabilities);
                previous_installer.release_date = analyser.last_modified;
                if analyser.display_name.is_some()
                    || analyser.display_publisher.is_some()
                    || analyser.display_version.is_some()
                    || analyser.upgrade_code.is_some()
                {
                    previous_installer.apps_and_features_entries =
                        Some(BTreeSet::from([AppsAndFeaturesEntry {
                            display_name: analyser.display_name.clone(),
                            publisher: analyser.display_publisher.clone(),
                            display_version: analyser
                                .display_version
                                .as_ref()
                                .filter(|&version| *version != self.package_version.to_string())
                                .cloned(),
                            product_code: analyser.product_code.clone(),
                            upgrade_code: analyser.upgrade_code.clone(),
                            ..AppsAndFeaturesEntry::default()
                        }]));
                }
                previous_installer
                    .elevation_requirement
                    .clone_from(&analyser.elevation_requirement);
                if let Some(install_location) = &analyser.default_install_location {
                    previous_installer.installation_metadata = Some(InstallationMetadata {
                        default_install_location: Some(install_location.clone()),
                        ..InstallationMetadata::default()
                    });
                }
                previous_installer
            })
            .collect::<Vec<_>>();

        manifests.installer_manifest.minimum_os_version = manifests
            .installer_manifest
            .minimum_os_version
            .filter(|minimum_os_version| *minimum_os_version != MinimumOSVersion::removable());
        manifests.installer_manifest.installers = installers;
        manifests
            .installer_manifest
            .reorder_keys(&self.package_identifier, &self.package_version);
        manifests.installer_manifest.installers.sort_unstable();

        let mut github_values = match github_values {
            Some(future) => Some(future.await?),
            None => None,
        };

        manifests
            .default_locale_manifest
            .update(&self.package_version, &mut github_values);

        manifests
            .locale_manifests
            .iter_mut()
            .for_each(|locale| locale.update(&self.package_version, &github_values));

        manifests.version_manifest.update(&self.package_version);

        let package_path = get_package_path(&self.package_identifier, Some(&self.package_version));
        let mut changes = PRChangesBuilder::default()
            .package_identifier(&self.package_identifier)
            .manifests(manifests)
            .package_path(&package_path)
            .created_with(&self.created_with)
            .build()?
            .create()?;

        let submit_option = prompt_submit_option(
            &mut changes,
            self.submit,
            &self.package_identifier,
            &self.package_version,
            self.dry_run,
        )?;

        if let Some(output) = self.output.map(|out| out.join(package_path)) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        if submit_option == SubmitOption::Exit {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {} version {}",
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
            &UpdateState::get(&self.package_version, Some(&versions), Some(latest_version)),
        );
        let changes = changes
            .iter()
            .map(|(path, content)| FileAddition {
                contents: Base64String(base64ct::Base64::encode_string(content.as_bytes())),
                path,
            })
            .collect::<Vec<_>>();
        let _commit_url = github
            .create_commit(
                &pull_request_branch.id,
                &pull_request_branch
                    .target
                    .map(|object| object.oid.0)
                    .unwrap(),
                &commit_title,
                Some(changes),
                None,
            )
            .await?;
        let pull_request_url = github
            .create_pull_request(
                &winget_pkgs.id,
                &fork.id,
                &format!("{current_user}:{}", pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &get_pull_request_body(
                    self.resolves,
                    None,
                    self.created_with,
                    self.created_with_url,
                ),
            )
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
    zip: &Option<Zip<R>>,
) -> Option<BTreeSet<NestedInstallerFiles>> {
    let relative_paths = nested_installer_files
        .into_iter()
        .filter_map(|nested_installer_files| {
            if let Some(zip) = zip {
                return if zip
                    .identified_files
                    .contains(&nested_installer_files.relative_file_path.normalize())
                {
                    Some(nested_installer_files)
                } else {
                    zip.identified_files
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
