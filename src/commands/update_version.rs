use std::collections::BTreeSet;
use std::mem;
use std::num::{NonZeroU32, NonZeroU8};
use std::time::Duration;

use base64ct::Encoding;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use crossterm::style::Stylize;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar};
use reqwest::Client;

use crate::commands::utils::{
    prompt_existing_pull_request, prompt_submit_option, reorder_keys, write_changes_to_dir,
    SubmitOption,
};
use crate::credential::{get_default_headers, handle_token};
use crate::download_file::{download_urls, process_files};
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::graphql::create_commit::{Base64String, FileAddition};
use crate::github::utils::{
    get_branch_name, get_commit_title, get_package_path, get_pull_request_body,
};
use crate::manifest::{build_manifest_string, Manifest};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, Installer, NestedInstallerFiles, Scope, UpgradeBehavior,
};
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::match_installers::match_installers;
use crate::types::installer_type::InstallerType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::path::NormalizePath;
use crate::types::urls::url::Url;
use crate::update_state::UpdateState;
use crate::zip::Zip;

#[derive(Parser)]
pub struct UpdateVersion {
    /// The package's unique identifier
    #[arg(id = "package_identifier", short = 'i', long = "identifier")]
    identifier: PackageIdentifier,

    /// The package's version
    #[arg(id = "package_version", short = 'v', long = "version")]
    version: PackageVersion,

    /// The list of package installers
    #[arg(short, long, num_args = 1.., required = true)]
    urls: Vec<Url>,

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

    /// List of issues that updating this package would resolve
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

    /// Automatically submit a pull request
    #[arg(short, long)]
    submit: bool,

    /// Name of external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH")]
    created_with: Option<String>,

    /// URL to external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH_URL")]
    created_with_url: Option<Url>,

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

        let existing_pr_future = github.get_existing_pull_request(&self.identifier, &self.version);

        let versions = github
            .get_versions(&get_package_path(&self.identifier, None))
            .await
            .wrap_err_with(|| {
                format!(
                    "{} does not exist in {WINGET_PKGS_FULL_NAME}",
                    self.identifier
                )
            })?;

        let latest_version = versions.iter().max().unwrap();
        println!("Latest version of {}: {latest_version}", self.identifier);

        if let Some(pull_request) = existing_pr_future.await? {
            if !self.dry_run
                && !prompt_existing_pull_request(&self.identifier, &self.version, &pull_request)?
            {
                return Ok(());
            }
        }

        let manifests = github.get_manifests(&self.identifier, latest_version);
        let multi_progress = MultiProgress::new();
        let files = stream::iter(download_urls(&client, self.urls, &multi_progress))
            .buffer_unordered(self.concurrent_downloads.get() as usize)
            .try_collect::<Vec<_>>()
            .await?;
        multi_progress.clear()?;
        let github_values = files
            .iter()
            .find(|download| download.url.host_str() == Some("github.com"))
            .map(|download| {
                let parts = download.url.path_segments().unwrap().collect::<Vec<_>>();
                github.get_all_values(
                    parts[0].to_owned(),
                    parts[1].to_owned(),
                    parts[4..parts.len() - 1].join("/"),
                )
            });
        let manifests = manifests.await?;
        let download_results = process_files(files).await?;
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
        let mut previous_installer_manifest = manifests.installer_manifest;
        let previous_installers = mem::take(&mut previous_installer_manifest.installers)
            .into_iter()
            .map(|installer| Installer {
                installer_type: previous_installer_manifest
                    .installer_type
                    .or(installer.installer_type),
                nested_installer_type: previous_installer_manifest
                    .nested_installer_type
                    .or(installer.nested_installer_type),
                scope: previous_installer_manifest.scope.or(installer.scope),
                ..installer
            })
            .collect::<Vec<_>>();
        let matched_installers = match_installers(previous_installers, &installer_results);
        let installers = matched_installers
            .into_iter()
            .map(|(previous_installer, new_installer)| {
                let analyser = download_results.get(&new_installer.installer_url).unwrap();
                Installer {
                    installer_locale: analyser
                        .product_language
                        .clone()
                        .or(previous_installer.installer_locale),
                    platform: analyser.platform.clone().or(previous_installer.platform),
                    minimum_os_version: analyser
                        .minimum_os_version
                        .clone()
                        .or(previous_installer.minimum_os_version)
                        .filter(|minimum_os_version| &**minimum_os_version != "10.0.0.0"),
                    architecture: previous_installer.architecture,
                    installer_type: match previous_installer.installer_type {
                        Some(InstallerType::Portable) => previous_installer.installer_type,
                        _ => match new_installer.installer_type {
                            Some(InstallerType::Portable) => previous_installer.installer_type,
                            _ => new_installer.installer_type,
                        },
                    },
                    nested_installer_type: analyser
                        .zip
                        .as_ref()
                        .and_then(|zip| zip.nested_installer_type)
                        .or(previous_installer.nested_installer_type)
                        .or(previous_installer_manifest.nested_installer_type),
                    nested_installer_files: previous_installer
                        .nested_installer_files
                        .or_else(|| previous_installer_manifest.nested_installer_files.clone())
                        .or_else(|| {
                            analyser
                                .zip
                                .as_ref()
                                .and_then(|zip| zip.nested_installer_files.clone())
                        })
                        .and_then(|nested_installer_files| {
                            validate_relative_paths(nested_installer_files, &analyser.zip)
                        }),
                    scope: new_installer.scope.or(previous_installer.scope),
                    installer_url: new_installer.installer_url.clone(),
                    installer_sha_256: analyser.installer_sha_256.clone(),
                    signature_sha_256: analyser.signature_sha_256.clone(),
                    install_modes: previous_installer.install_modes,
                    installer_switches: previous_installer.installer_switches,
                    installer_success_codes: previous_installer.installer_success_codes,
                    upgrade_behavior: UpgradeBehavior::get(analyser.installer_type)
                        .or(previous_installer.upgrade_behavior),
                    commands: previous_installer.commands,
                    protocols: previous_installer.protocols,
                    file_extensions: previous_installer.file_extensions,
                    package_family_name: analyser.package_family_name.clone(),
                    product_code: analyser.product_code.clone(),
                    release_date: analyser.last_modified,
                    apps_and_features_entries: analyser.msi.as_ref().map(|msi| {
                        BTreeSet::from([AppsAndFeaturesEntry {
                            display_name: if msi.product_name
                                == manifests.default_locale_manifest.package_name.as_str()
                            {
                                None
                            } else {
                                Some(msi.product_name.clone())
                            },
                            display_version: if msi.product_version == self.version.to_string() {
                                None
                            } else {
                                Some(msi.product_version.clone())
                            },
                            upgrade_code: Some(msi.upgrade_code.clone()),
                            ..AppsAndFeaturesEntry::default()
                        }])
                    }),
                    ..previous_installer
                }
            })
            .collect::<BTreeSet<_>>();

        let mut installer_manifest = reorder_keys(
            self.identifier.clone(),
            self.version.clone(),
            installers,
            previous_installer_manifest,
        );
        installer_manifest.minimum_os_version = installer_manifest
            .minimum_os_version
            .filter(|minimum_os_version| &**minimum_os_version != "10.0.0.0");
        let previous_default_locale_manifest = manifests.default_locale_manifest;
        let mut github_values = match github_values {
            Some(future) => Some(future.await?),
            None => None,
        };
        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: self.identifier.clone(),
            package_version: self.version.clone(),
            publisher_url: previous_default_locale_manifest.publisher_url.or_else(|| {
                github_values
                    .as_mut()
                    .map(|values| mem::take(&mut values.publisher_url))
            }),
            license: github_values
                .as_mut()
                .and_then(|values| mem::take(&mut values.license))
                .unwrap_or(previous_default_locale_manifest.license),
            license_url: github_values
                .as_mut()
                .and_then(|values| mem::take(&mut values.license_url))
                .or(previous_default_locale_manifest.license_url),
            tags: previous_default_locale_manifest.tags.or_else(|| {
                github_values
                    .as_mut()
                    .and_then(|values| mem::take(&mut values.topics))
            }),
            release_notes: github_values
                .as_mut()
                .and_then(|values| mem::take(&mut values.release_notes)),
            release_notes_url: github_values
                .as_ref()
                .map(|values| values.release_notes_url.clone()),
            manifest_version: ManifestVersion::default(),
            ..previous_default_locale_manifest
        };
        let version_manifest = VersionManifest {
            package_identifier: self.identifier.clone(),
            package_version: self.version.clone(),
            manifest_version: ManifestVersion::default(),
            ..manifests.version_manifest
        };

        let full_package_path = get_package_path(&self.identifier, Some(&self.version));
        let mut changes = {
            let mut path_content_map = Vec::new();
            path_content_map.push((
                format!("{full_package_path}/{}.installer.yaml", self.identifier),
                build_manifest_string(
                    &Manifest::Installer(&installer_manifest),
                    &self.created_with,
                )?,
            ));
            path_content_map.push((
                format!(
                    "{full_package_path}/{}.locale.{}.yaml",
                    self.identifier, version_manifest.default_locale
                ),
                build_manifest_string(
                    &Manifest::DefaultLocale(&default_locale_manifest),
                    &self.created_with,
                )?,
            ));
            manifests
                .locale_manifests
                .into_iter()
                .map(|locale_manifest| LocaleManifest {
                    package_version: self.version.clone(),
                    release_notes_url: github_values
                        .as_ref()
                        .map(|values| values.release_notes_url.clone()),
                    manifest_version: ManifestVersion::default(),
                    ..locale_manifest
                })
                .for_each(|locale_manifest| {
                    if let Ok(yaml) = build_manifest_string(
                        &Manifest::Locale(&locale_manifest),
                        &self.created_with,
                    ) {
                        path_content_map.push((
                            format!(
                                "{full_package_path}/{}.locale.{}.yaml",
                                self.identifier, locale_manifest.package_locale
                            ),
                            yaml,
                        ));
                    }
                });
            path_content_map.push((
                format!("{full_package_path}/{}.yaml", self.identifier),
                build_manifest_string(&Manifest::Version(&version_manifest), &self.created_with)?,
            ));
            path_content_map
        };

        let submit_option = prompt_submit_option(
            &mut changes,
            self.submit,
            &self.identifier,
            &self.version,
            self.dry_run,
        )?;

        if let Some(output) = self.output.map(|out| out.join(full_package_path)) {
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
            self.identifier, self.version
        ));
        pr_progress.enable_steady_tick(Duration::from_millis(50));

        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs(None).await?;
        let fork = github.get_winget_pkgs(Some(&current_user)).await?;
        let branch_name = get_branch_name(&self.identifier, &self.version);
        let pull_request_branch = github
            .create_branch(&fork.id, &branch_name, &winget_pkgs.default_branch_oid.0)
            .await?;
        let commit_title = get_commit_title(
            &self.identifier,
            &self.version,
            &UpdateState::get(&self.version, Some(&versions), Some(latest_version)),
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

fn validate_relative_paths(
    nested_installer_files: BTreeSet<NestedInstallerFiles>,
    zip: &Option<Zip>,
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
                        .find(|file_path| {
                            file_path.file_name()
                                == nested_installer_files.relative_file_path.file_name()
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
