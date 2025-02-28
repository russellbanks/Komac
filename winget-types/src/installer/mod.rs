mod apps_and_features_entry;
mod architecture;
mod command;
mod dependencies;
mod elevation_requirement;
mod expected_return_codes;
mod file_extension;
mod install_modes;
mod installation_metadata;
mod installer_return_code;
mod installer_type;
mod markets;
mod minimum_os_version;
mod nested;
mod platform;
mod protocol;
mod repair_behavior;
mod return_response;
mod scope;
pub mod switches;
mod unsupported_arguments;
mod unsupported_os_architectures;
mod upgrade_behavior;

use std::collections::BTreeSet;

use chrono::NaiveDate;
use const_format::formatc;
use itertools::Itertools;
use nested::installer_type::NestedInstallerType;
use package_family_name::PackageFamilyName;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use crate::installer::{
    apps_and_features_entry::AppsAndFeaturesEntry,
    architecture::{Architecture, VALID_FILE_EXTENSIONS},
    command::Command,
    dependencies::Dependencies,
    elevation_requirement::ElevationRequirement,
    expected_return_codes::ExpectedReturnCodes,
    file_extension::FileExtension,
    install_modes::InstallModes,
    installation_metadata::InstallationMetadata,
    installer_return_code::{InstallerReturnCode, InstallerSuccessCode},
    installer_type::InstallerType,
    markets::Markets,
    minimum_os_version::MinimumOSVersion,
    nested::installer_files::NestedInstallerFiles,
    platform::Platform,
    protocol::Protocol,
    repair_behavior::RepairBehavior,
    scope::Scope,
    switches::InstallerSwitches,
    unsupported_arguments::UnsupportedArguments,
    unsupported_os_architectures::UnsupportedOSArchitecture,
    upgrade_behavior::UpgradeBehavior,
};
use crate::{
    shared::{
        LanguageTag, ManifestType, ManifestVersion, PackageIdentifier, PackageVersion,
        Sha256String, url::DecodedUrl,
    },
    traits::Manifest,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InstallerManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub channel: Option<String>,
    #[serde(rename = "InstallerLocale")]
    pub locale: Option<LanguageTag>,
    pub platform: Option<Platform>,
    #[serde(rename = "MinimumOSVersion")]
    pub minimum_os_version: Option<MinimumOSVersion>,
    #[serde(rename = "InstallerType")]
    pub r#type: Option<InstallerType>,
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub scope: Option<Scope>,
    pub install_modes: Option<InstallModes>,
    #[serde(rename = "InstallerSwitches")]
    pub switches: Option<InstallerSwitches>,
    #[serde(rename = "InstallerSuccessCodes")]
    pub success_codes: Option<BTreeSet<InstallerSuccessCode>>,
    pub expected_return_codes: Option<BTreeSet<ExpectedReturnCodes>>,
    pub upgrade_behavior: Option<UpgradeBehavior>,
    pub commands: Option<BTreeSet<Command>>,
    pub protocols: Option<BTreeSet<Protocol>>,
    pub file_extensions: Option<BTreeSet<FileExtension>>,
    pub dependencies: Option<Dependencies>,
    pub package_family_name: Option<PackageFamilyName>,
    pub product_code: Option<String>,
    pub capabilities: Option<BTreeSet<String>>,
    pub restricted_capabilities: Option<BTreeSet<String>>,
    pub markets: Option<Markets>,
    #[serde(rename = "InstallerAbortsTerminal")]
    pub aborts_terminal: Option<bool>,
    pub release_date: Option<NaiveDate>,
    pub install_location_required: Option<bool>,
    pub require_explicit_upgrade: Option<bool>,
    pub display_install_warnings: Option<bool>,
    #[serde(rename = "UnsupportedOSArchitectures")]
    pub unsupported_os_architectures: Option<UnsupportedOSArchitecture>,
    pub unsupported_arguments: Option<UnsupportedArguments>,
    pub apps_and_features_entries: Option<Vec<AppsAndFeaturesEntry>>,
    pub elevation_requirement: Option<ElevationRequirement>,
    pub installation_metadata: Option<InstallationMetadata>,
    pub download_command_prohibited: Option<bool>,
    pub repair_behavior: Option<RepairBehavior>,
    pub archive_binaries_depend_on_path: Option<bool>,
    pub installers: Vec<Installer>,
    pub manifest_type: ManifestType,
    #[serde(default)]
    pub manifest_version: ManifestVersion,
}

impl Manifest for InstallerManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.installer.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::Installer;
}

impl InstallerManifest {
    pub fn reorder_keys(
        &mut self,
        package_identifier: &PackageIdentifier,
        package_version: &PackageVersion,
    ) {
        fn reorder_key<T>(
            installers: &mut [Installer],
            get_installer_key: impl Fn(&mut Installer) -> &mut Option<T>,
            root_key: &mut Option<T>,
        ) where
            T: PartialEq,
        {
            if let Ok(value) = installers
                .iter_mut()
                .map(&get_installer_key)
                .all_equal_value()
            {
                if value.is_some() {
                    *root_key = value.take();
                    installers
                        .iter_mut()
                        .for_each(|installer| *get_installer_key(installer) = None);
                }
            }
        }

        fn reorder_struct_key<T, R>(
            installers: &mut [Installer],
            get_installer_struct: impl Fn(&mut Installer) -> &mut Option<T>,
            get_struct_key: impl Fn(&mut T) -> &mut Option<R>,
            root_struct: &mut Option<T>,
        ) where
            T: Default,
            R: PartialEq,
        {
            if let Ok(Some(common_value)) = installers
                .iter_mut()
                .map(&get_installer_struct)
                .map(|r#struct| r#struct.as_mut().map(&get_struct_key))
                .all_equal_value()
            {
                if common_value.is_some() {
                    *get_struct_key(root_struct.get_or_insert_default()) = common_value.take();

                    installers
                        .iter_mut()
                        .filter_map(|installer| get_installer_struct(installer).as_mut())
                        .for_each(|r#struct| *get_struct_key(r#struct) = None);
                }
            }
        }

        macro_rules! reorder_root_keys {
            ($($field:ident),*) => {
                $(
                    reorder_key(&mut self.installers, |installer| &mut installer.$field, &mut self.$field);
                )*
            };
        }

        macro_rules! reorder_struct_key {
            ($struct:ident, $( $field:ident ),*) => {
                $(
                    reorder_struct_key(&mut self.installers, |installer| &mut installer.$struct, |s| &mut s.$field, &mut self.$struct);
                )*
                self.installers.iter_mut().for_each(|installer| {
                    if let Some(r#struct) = &mut installer.$struct {
                        if !r#struct.is_any_some() {
                            installer.$struct = None;
                        }
                    }
                });
            };
        }

        self.package_identifier.clone_from(package_identifier);
        self.package_version.clone_from(package_version);
        reorder_root_keys!(
            locale,
            platform,
            minimum_os_version,
            r#type,
            nested_installer_type,
            nested_installer_files,
            scope,
            install_modes,
            success_codes,
            expected_return_codes,
            upgrade_behavior,
            commands,
            protocols,
            file_extensions,
            dependencies,
            package_family_name,
            product_code,
            capabilities,
            restricted_capabilities,
            markets,
            aborts_terminal,
            release_date,
            install_location_required,
            require_explicit_upgrade,
            display_install_warnings,
            unsupported_os_architectures,
            unsupported_arguments,
            apps_and_features_entries,
            elevation_requirement,
            installation_metadata,
            download_command_prohibited,
            repair_behavior,
            archive_binaries_depend_on_path
        );
        reorder_struct_key!(
            switches,
            silent,
            silent_with_progress,
            interactive,
            install_location,
            log,
            silent_with_progress,
            upgrade,
            custom,
            repair
        );

        if self
            .apps_and_features_entries
            .as_ref()
            .is_some_and(|entries| !entries.iter().any(AppsAndFeaturesEntry::is_any_some))
        {
            self.apps_and_features_entries = None;
        }

        self.manifest_version = ManifestVersion::default();

        self.installers.sort_unstable();
        self.installers.dedup();
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Installer {
    #[serde(rename = "InstallerLocale")]
    pub locale: Option<LanguageTag>,
    pub platform: Option<Platform>,
    #[serde(rename = "MinimumOSVersion")]
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Architecture,
    #[serde(rename = "InstallerType")]
    pub r#type: Option<InstallerType>,
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub scope: Option<Scope>,
    #[serde(rename = "InstallerUrl")]
    pub url: DecodedUrl,
    #[serde(rename = "InstallerSha256")]
    pub sha_256: Sha256String,
    pub signature_sha_256: Option<Sha256String>,
    pub install_modes: Option<InstallModes>,
    #[serde(rename = "InstallerSwitches")]
    pub switches: Option<InstallerSwitches>,
    #[serde(rename = "InstallerSuccessCodes")]
    pub success_codes: Option<BTreeSet<InstallerSuccessCode>>,
    pub expected_return_codes: Option<BTreeSet<ExpectedReturnCodes>>,
    pub upgrade_behavior: Option<UpgradeBehavior>,
    pub commands: Option<BTreeSet<Command>>,
    pub protocols: Option<BTreeSet<Protocol>>,
    pub file_extensions: Option<BTreeSet<FileExtension>>,
    pub dependencies: Option<Dependencies>,
    pub package_family_name: Option<PackageFamilyName>,
    pub product_code: Option<String>,
    pub capabilities: Option<BTreeSet<String>>,
    pub restricted_capabilities: Option<BTreeSet<String>>,
    pub markets: Option<Markets>,
    #[serde(rename = "InstallerAbortsTerminal")]
    pub aborts_terminal: Option<bool>,
    pub release_date: Option<NaiveDate>,
    pub install_location_required: Option<bool>,
    pub require_explicit_upgrade: Option<bool>,
    pub display_install_warnings: Option<bool>,
    #[serde(rename = "UnsupportedOSArchitectures")]
    pub unsupported_os_architectures: Option<UnsupportedOSArchitecture>,
    pub unsupported_arguments: Option<UnsupportedArguments>,
    pub apps_and_features_entries: Option<Vec<AppsAndFeaturesEntry>>,
    pub elevation_requirement: Option<ElevationRequirement>,
    pub installation_metadata: Option<InstallationMetadata>,
    pub download_command_prohibited: Option<bool>,
    pub repair_behavior: Option<RepairBehavior>,
    pub archive_binaries_depend_on_path: Option<bool>,
}

impl Installer {
    #[must_use]
    pub fn merge_with(mut self, other: Self) -> Self {
        macro_rules! merge_fields {
            ($self:ident, $other:ident, $( $field:ident ),* ) => {
                $(
                    if $self.$field.is_none() {
                        $self.$field = $other.$field;
                    }
                )*
            }
        }

        if let (Some(custom), Some(other_custom)) = (
            self.switches
                .as_mut()
                .and_then(|switches| switches.custom.as_mut()),
            other
                .switches
                .as_ref()
                .and_then(|switches| switches.custom.as_ref()),
        ) {
            for part in other_custom {
                if !custom.contains(part) {
                    custom.push(part.clone());
                }
            }
        }

        merge_fields!(
            self,
            other,
            locale,
            platform,
            minimum_os_version,
            r#type,
            nested_installer_type,
            nested_installer_files,
            scope,
            install_modes,
            switches,
            success_codes,
            expected_return_codes,
            upgrade_behavior,
            commands,
            protocols,
            file_extensions,
            dependencies,
            package_family_name,
            product_code,
            capabilities,
            restricted_capabilities,
            markets,
            aborts_terminal,
            require_explicit_upgrade,
            display_install_warnings,
            unsupported_os_architectures,
            unsupported_arguments,
            elevation_requirement,
            installation_metadata,
            download_command_prohibited,
            repair_behavior,
            archive_binaries_depend_on_path
        );

        self
    }
}
