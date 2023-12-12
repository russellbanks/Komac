use crate::credential::{get_default_headers, handle_token};
use crate::default_locale_manifest::DefaultLocaleManifest;
use crate::download_file::{download_urls, process_files};
use crate::file_analyser::get_upgrade_behavior;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::github_utils::{
    get_branch_name, get_commit_title, get_full_package_path, get_package_path,
    get_pull_request_body,
};
use crate::graphql::create_commit::FileAddition;
use crate::installer_manifest::{AppsAndFeaturesEntry, Installer, InstallerManifest};
use crate::iterable_extensions::IterableExt;
use crate::locale_manifest::LocaleManifest;
use crate::manifest::{build_manifest_string, print_changes, Manifest};
use crate::match_installers::match_installers;
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::update_state::get_update_state;
use crate::url_utils::find_scope;
use crate::version_manifest::VersionManifest;
use base64ct::Encoding;
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use crossterm::style::Stylize;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use inquire::Confirm;
use percent_encoding::percent_decode_str;
use reqwest::{Client, Url};
use std::collections::BTreeSet;
use std::num::NonZeroU8;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Parser)]
pub struct Update {
    #[arg(id = "package_identifier", short = 'i', long = "identifier")]
    identifier: PackageIdentifier,

    #[arg(id = "package_version", short = 'v', long = "version")]
    version: PackageVersion,

    #[arg(short, long, num_args=1.., required = true)]
    urls: Vec<Url>,

    // Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

    #[arg(short, long)]
    submit: bool,

    #[arg(short, long, env = "OUTPUT_DIRECTORY", value_hint = clap::ValueHint::DirPath)]
    output: Option<PathBuf>,

    /// GitHub personal access token with the public_repo scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Update {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        let github = GitHub::new(token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let versions = github
            .get_versions(&get_package_path(&self.identifier))
            .await
            .wrap_err_with(|| {
                format!(
                    "{} does not exist in {}",
                    self.identifier, WINGET_PKGS_FULL_NAME
                )
            })?;

        let latest_version = versions.iter().max().unwrap();
        println!("Latest version of {}: {latest_version}", &self.identifier);
        let manifests = github.get_manifests(&self.identifier, latest_version);
        let multi_progress = MultiProgress::new();
        let files = stream::iter(download_urls(&client, &self.urls, &multi_progress))
            .buffer_unordered(self.concurrent_downloads.get() as usize)
            .try_collect::<Vec<_>>()
            .await?;
        multi_progress.clear()?;
        let github_values = files
            .iter()
            .filter_map(|download| Url::parse(download.url).ok())
            .find(|url| url.host_str() == Some("github.com"))
            .map(|url| {
                let parts = url.path_segments().unwrap().collect::<Vec<_>>();
                github.get_all_values(
                    parts[0].to_owned(),
                    parts[1].to_owned(),
                    percent_decode_str(parts[parts.len() - 2])
                        .decode_utf8()
                        .unwrap()
                        .into_owned(),
                )
            });
        let download_results = process_files(files).await?;
        let installer_results = download_results
            .iter()
            .map(|(url, download)| Installer {
                architecture: download.architecture,
                installer_type: Some(download.installer_type),
                scope: find_scope(url),
                installer_url: Url::parse(url).unwrap(),
                ..Installer::default()
            })
            .collect::<Vec<_>>();
        let manifests = manifests.await?;
        let previous_installer_manifest = manifests.installer_manifest;
        let matched_installers = match_installers(
            &previous_installer_manifest
                .installers
                .clone()
                .into_iter()
                .map(|installer| Installer {
                    installer_type: previous_installer_manifest
                        .installer_type
                        .or(installer.installer_type),
                    scope: previous_installer_manifest.scope.or(installer.scope),
                    ..installer
                })
                .collect::<Vec<_>>(),
            &installer_results,
        );
        let installers = matched_installers
            .into_iter()
            .map(|(previous_installer, new_installer)| {
                let download = &download_results[new_installer.installer_url.as_str()];
                Installer {
                    installer_locale: download
                        .msi
                        .as_ref()
                        .map(|msi| msi.product_language.to_owned())
                        .or(previous_installer.installer_locale)
                        .or(previous_installer_manifest.installer_locale.to_owned()),
                    platform: download
                        .msix
                        .as_ref()
                        .map(|msix| BTreeSet::from([msix.target_device_family]))
                        .or(previous_installer.platform)
                        .or(previous_installer_manifest.platform.to_owned()),
                    minimum_os_version: download
                        .msix
                        .as_ref()
                        .map(|msix| msix.min_version.to_owned())
                        .or(previous_installer.minimum_os_version)
                        .or(previous_installer_manifest.minimum_os_version.to_owned())
                        .filter(|minimum_os_version| minimum_os_version.deref() != "10.0.0.0"),
                    architecture: previous_installer.architecture,
                    installer_type: new_installer.installer_type,
                    scope: new_installer
                        .scope
                        .or(previous_installer.scope)
                        .or(previous_installer_manifest.scope),
                    installer_url: Url::parse(
                        &percent_decode_str(new_installer.installer_url.as_str())
                            .decode_utf8()
                            .unwrap_or_default(),
                    )
                    .unwrap_or(new_installer.installer_url),
                    installer_sha_256: download.installer_sha_256.to_owned(),
                    signature_sha_256: download
                        .msix
                        .as_ref()
                        .map(|msix| msix.signature_sha_256.to_owned())
                        .or(download
                            .msix_bundle
                            .as_ref()
                            .map(|msix_bundle| msix_bundle.signature_sha_256.to_owned())),
                    install_modes: previous_installer
                        .install_modes
                        .or(previous_installer_manifest.install_modes.to_owned()),
                    installer_switches: previous_installer
                        .installer_switches
                        .or(previous_installer_manifest.installer_switches.to_owned()),
                    installer_success_codes: previous_installer.installer_success_codes.or(
                        previous_installer_manifest
                            .installer_success_codes
                            .to_owned(),
                    ),
                    upgrade_behavior: get_upgrade_behavior(&download.installer_type)
                        .or(previous_installer.upgrade_behavior)
                        .or(previous_installer_manifest.upgrade_behavior),
                    commands: previous_installer
                        .commands
                        .or(previous_installer_manifest.commands.to_owned()),
                    protocols: previous_installer
                        .protocols
                        .or(previous_installer_manifest.protocols.to_owned()),
                    file_extensions: previous_installer
                        .file_extensions
                        .or(previous_installer_manifest.file_extensions.to_owned()),
                    package_family_name: download
                        .msix
                        .as_ref()
                        .map(|msix| msix.package_family_name.to_owned()),
                    product_code: download.msi.as_ref().map(|msi| msi.product_code.to_owned()),
                    release_date: download.last_modified,
                    apps_and_features_entries: download.msi.as_ref().map(|msi| {
                        BTreeSet::from([AppsAndFeaturesEntry {
                            display_name: if msi.product_name
                                != manifests.default_locale_manifest.package_name.as_str()
                            {
                                Some(msi.product_name.to_owned())
                            } else {
                                None
                            },
                            display_version: if msi.product_version != self.version.to_string() {
                                Some(msi.product_version.to_owned())
                            } else {
                                None
                            },
                            upgrade_code: Some(msi.upgrade_code.to_owned()),
                            ..AppsAndFeaturesEntry::default()
                        }])
                    }),
                    ..previous_installer
                }
            })
            .collect::<BTreeSet<_>>();

        let installer_manifest = set_root_keys(
            self.identifier.clone(),
            self.version.clone(),
            installers,
            previous_installer_manifest,
        );
        let previous_default_locale_manifest = manifests.default_locale_manifest;
        let github_values = match github_values {
            Some(future) => Some(future.await),
            None => None,
        }
        .transpose()?;
        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: self.identifier.clone(),
            package_version: self.version.clone(),
            publisher_url: previous_default_locale_manifest
                .publisher_url
                .or(github_values
                    .as_ref()
                    .map(|values| values.publisher_url.to_owned())),
            license: github_values
                .as_ref()
                .and_then(|values| values.license.to_owned())
                .unwrap_or(previous_default_locale_manifest.license),
            license_url: github_values
                .as_ref()
                .and_then(|values| values.license_url.to_owned())
                .or(previous_default_locale_manifest.license_url),
            release_notes: github_values
                .as_ref()
                .and_then(|values| values.release_notes.to_owned()),
            release_notes_url: github_values.map(|values| values.release_notes_url),
            manifest_version: ManifestVersion::default(),
            ..previous_default_locale_manifest
        };
        let version_manifest = VersionManifest {
            package_identifier: self.identifier.clone(),
            package_version: self.version.clone(),
            manifest_version: ManifestVersion::default(),
            ..manifests.version_manifest
        };

        let changes = {
            let full_package_path = get_full_package_path(&self.identifier, &self.version);
            let mut path_content_map = Vec::new();
            path_content_map.push((
                format!("{full_package_path}/{}.installer.yaml", self.identifier),
                build_manifest_string(Manifest::Installer(&installer_manifest))?,
            ));
            path_content_map.push((
                format!(
                    "{full_package_path}/{}.locale.{}.yaml",
                    self.identifier, version_manifest.default_locale
                ),
                build_manifest_string(Manifest::DefaultLocale(&default_locale_manifest))?,
            ));
            manifests
                .locale_manifests
                .into_iter()
                .map(|locale_manifest| LocaleManifest {
                    manifest_version: ManifestVersion::default(),
                    ..locale_manifest
                })
                .for_each(|locale_manifest| {
                    if let Ok(yaml) = build_manifest_string(Manifest::Locale(&locale_manifest)) {
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
                build_manifest_string(Manifest::Version(&version_manifest))?,
            ));
            path_content_map
        };

        print_changes(&changes);

        if let Some(output) = self.output {
            stream::iter(
                changes
                    .iter()
                    .map(|(_, content)| fs::write(&output, content)),
            )
            .buffer_unordered(2)
            .try_collect::<Vec<_>>()
            .await?;
            println!(
                "{} written all manifest files to {}",
                "Successfully".green(),
                output.to_str().unwrap_or("the given directory")
            );
        }

        let should_remove_manifest = if self.submit {
            true
        } else {
            Confirm::new(&format!(
                "Would you like to make a pull request for {} {}?",
                self.identifier, self.version
            ))
            .prompt()?
        };
        if !should_remove_manifest {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let create_pr_progress = Arc::new(Mutex::new(
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template("{spinner:.green} {msg}")?)
                .with_message(format!(
                    "Creating a pull request for {} version {}",
                    self.identifier, self.version
                )),
        ));

        // Spawn a new thread that ticks the progress bar every 50 milliseconds
        let pb = create_pr_progress.clone();
        let update_spinner_thread = tokio::spawn(async move {
            loop {
                pb.lock().await.tick();
                sleep(Duration::from_millis(50)).await;
            }
        });

        let current_user = github.get_username().await?;
        let winget_pkgs = github.get_winget_pkgs().await?;
        let fork_id = github.get_winget_pkgs_fork_id(&current_user).await?;
        let branch_name = get_branch_name(&self.identifier, &self.version);
        let pull_request_branch = github
            .create_branch(&fork_id, &branch_name, &winget_pkgs.default_branch_oid)
            .await?;
        let commit_title = get_commit_title(
            &self.identifier,
            &self.version,
            get_update_state(&self.version, &versions, latest_version),
        );
        let changes = changes
            .into_iter()
            .map(|(path, content)| FileAddition {
                contents: base64ct::Base64::encode_string(content.as_bytes()),
                path,
            })
            .collect::<Vec<_>>();
        let _commit_url = github
            .create_commit(
                &pull_request_branch.id,
                &pull_request_branch.head_sha,
                &commit_title,
                Some(changes),
                None,
            )
            .await?;
        let pull_request_url = github
            .create_pull_request(
                &winget_pkgs.id,
                &fork_id,
                &format!("{current_user}:{}", pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &get_pull_request_body(),
            )
            .await?;

        update_spinner_thread.abort();
        create_pr_progress.lock().await.finish_and_clear();

        println!(
            "{} created a pull request to {}",
            "Successfully".green(),
            WINGET_PKGS_FULL_NAME
        );
        println!("{}", pull_request_url.as_str());

        Ok(())
    }
}

fn remove_non_distinct_keys(installers: BTreeSet<Installer>) -> BTreeSet<Installer> {
    macro_rules! installer_key {
        ($item: expr, $field: ident) => {
            installers
                .iter()
                .single_or_else($item.$field, |installer| installer.$field.to_owned())
                .flatten()
        };
    }
    installers
        .clone()
        .into_iter()
        .map(|installer| Installer {
            installer_locale: installer_key!(installer, installer_locale),
            platform: installer_key!(installer, platform),
            minimum_os_version: installer_key!(installer, minimum_os_version),
            installer_type: installer_key!(installer, installer_type),
            nested_installer_type: installer_key!(installer, nested_installer_type),
            nested_installer_files: installer_key!(installer, nested_installer_files),
            scope: installer_key!(installer, scope),
            install_modes: installer_key!(installer, install_modes),
            installer_switches: installer_key!(installer, installer_switches),
            installer_success_codes: installer_key!(installer, installer_success_codes),
            expected_return_codes: installer_key!(installer, expected_return_codes),
            upgrade_behavior: installer_key!(installer, upgrade_behavior),
            commands: installer_key!(installer, commands),
            protocols: installer_key!(installer, protocols),
            file_extensions: installer_key!(installer, file_extensions),
            dependencies: installer_key!(installer, dependencies),
            package_family_name: installer_key!(installer, package_family_name),
            product_code: installer_key!(installer, product_code),
            capabilities: installer_key!(installer, capabilities),
            restricted_capabilities: installer_key!(installer, restricted_capabilities),
            markets: installer_key!(installer, markets),
            installer_aborts_terminal: installer_key!(installer, installer_aborts_terminal),
            release_date: installer_key!(installer, release_date),
            installer_location_required: installer_key!(installer, installer_location_required),
            require_explicit_upgrade: installer_key!(installer, require_explicit_upgrade),
            display_install_warnings: installer_key!(installer, display_install_warnings),
            unsupported_os_architectures: installer_key!(installer, unsupported_os_architectures),
            unsupported_arguments: installer_key!(installer, unsupported_arguments),
            apps_and_features_entries: installer_key!(installer, apps_and_features_entries),
            elevation_requirement: installer_key!(installer, elevation_requirement),
            installation_metadata: installer_key!(installer, installation_metadata),
            ..installer
        })
        .collect()
}

fn set_root_keys(
    package_identifier: PackageIdentifier,
    package_version: PackageVersion,
    installers: BTreeSet<Installer>,
    previous_installer_manifest: InstallerManifest,
) -> InstallerManifest {
    macro_rules! root_manifest_key {
        ($field:ident) => {
            installers
                .iter()
                .distinct_or_none(|installer| installer.$field.to_owned())
                .flatten()
        };
    }

    InstallerManifest {
        package_identifier,
        package_version,
        installer_locale: root_manifest_key!(installer_locale),
        platform: root_manifest_key!(platform),
        minimum_os_version: root_manifest_key!(minimum_os_version),
        installer_type: root_manifest_key!(installer_type),
        nested_installer_type: root_manifest_key!(nested_installer_type),
        nested_installer_files: root_manifest_key!(nested_installer_files),
        scope: root_manifest_key!(scope),
        install_modes: root_manifest_key!(install_modes),
        installer_switches: root_manifest_key!(installer_switches),
        installer_success_codes: root_manifest_key!(installer_success_codes),
        expected_return_codes: root_manifest_key!(expected_return_codes),
        upgrade_behavior: root_manifest_key!(upgrade_behavior),
        commands: root_manifest_key!(commands),
        protocols: root_manifest_key!(protocols),
        file_extensions: root_manifest_key!(file_extensions),
        dependencies: root_manifest_key!(dependencies),
        package_family_name: root_manifest_key!(package_family_name),
        product_code: root_manifest_key!(product_code),
        capabilities: root_manifest_key!(capabilities),
        restricted_capabilities: root_manifest_key!(restricted_capabilities),
        markets: root_manifest_key!(markets),
        installer_aborts_terminal: root_manifest_key!(installer_aborts_terminal),
        release_date: root_manifest_key!(release_date),
        installer_location_required: root_manifest_key!(installer_location_required),
        require_explicit_upgrade: root_manifest_key!(require_explicit_upgrade),
        display_install_warnings: root_manifest_key!(display_install_warnings),
        unsupported_os_architectures: root_manifest_key!(unsupported_os_architectures),
        unsupported_arguments: root_manifest_key!(unsupported_arguments),
        apps_and_features_entries: root_manifest_key!(apps_and_features_entries),
        elevation_requirement: root_manifest_key!(elevation_requirement),
        installation_metadata: root_manifest_key!(installation_metadata),
        installers: remove_non_distinct_keys(installers),
        manifest_version: ManifestVersion::default(),
        ..previous_installer_manifest
    }
}
