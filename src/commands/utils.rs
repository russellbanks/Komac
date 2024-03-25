use crate::github::graphql::get_existing_pull_request::PullRequest;
use crate::github::graphql::get_pull_request_from_branch::PullRequestState;
use crate::manifests::installer_manifest::{Installer, InstallerManifest};
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use camino::Utf8Path;
use color_eyre::Result;
use crossterm::style::Stylize;
use futures_util::{stream, StreamExt, TryStreamExt};
use inquire::Confirm;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::env;
use std::ops::Not;
use std::str::FromStr;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn prompt_existing_pull_request(
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    pull_request: &PullRequest,
) -> Result<bool> {
    println!(
        "There is already {} pull request for {identifier} {version} that was created on {} at {}",
        match pull_request.state {
            PullRequestState::Merged => "a merged",
            PullRequestState::Open => "an open",
            _ => "a closed",
        },
        pull_request.created_at.date_naive(),
        pull_request.created_at.time()
    );
    println!("{}", pull_request.url.as_str().blue());
    let proceed = if env::var("CI").is_ok_and(|ci| bool::from_str(&ci) == Ok(true)) {
        false
    } else {
        Confirm::new("Would you like to proceed?").prompt()?
    };
    Ok(proceed)
}

pub async fn write_changes_to_dir(changes: &[(String, String)], output: &Utf8Path) -> Result<()> {
    fs::create_dir_all(output).await?;
    stream::iter(changes.iter())
        .map(|(path, content)| async move {
            if let Some(file_name) = Utf8Path::new(path).file_name() {
                let mut file = File::create(output.join(file_name)).await?;
                file.write_all(content.as_bytes()).await?;
            }
            Ok::<(), color_eyre::eyre::Error>(())
        })
        .buffer_unordered(2)
        .try_collect()
        .await
}

pub fn reorder_keys(
    package_identifier: PackageIdentifier,
    package_version: PackageVersion,
    installers: BTreeSet<Installer>,
    installer_manifest: InstallerManifest,
) -> InstallerManifest {
    macro_rules! root_manifest_key {
        ($field:ident) => {
            installers
                .iter()
                .all(|installer| installer.$field.is_none())
                .then_some(installer_manifest.$field)
                .or_else(|| {
                    installers
                        .iter()
                        .map(|installer| installer.$field.as_ref())
                        .all_equal_value()
                        .ok()
                        .map(|value| value.cloned())
                })
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
        ..installer_manifest
    }
}

fn remove_non_distinct_keys(installers: BTreeSet<Installer>) -> BTreeSet<Installer> {
    macro_rules! installer_key {
        ($item: expr, $field: ident) => {
            installers
                .iter()
                .map(|installer| &installer.$field)
                .all_equal()
                .not()
                .then_some($item.$field)
                .flatten()
        };
    }
    installers
        .iter()
        .cloned()
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
