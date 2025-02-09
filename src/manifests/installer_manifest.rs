use std::collections::BTreeSet;
use std::num::NonZeroI64;

use crate::installers::utils::{
    RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64, RELATIVE_LOCAL_APP_DATA,
    RELATIVE_PROGRAM_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
    RELATIVE_SYSTEM_ROOT, RELATIVE_WINDOWS_DIR,
};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::ManifestTrait;
use crate::types::architecture::Architecture;
use crate::types::command::Command;
use crate::types::custom_switch::CustomSwitch;
use crate::types::file_extension::FileExtension;
use crate::types::install_modes::InstallModes;
use crate::types::installer_success_code::InstallerSuccessCode;
use crate::types::installer_switch::InstallerSwitch;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::protocol::Protocol;
use crate::types::sha_256::Sha256String;
use crate::types::silent_switch::SilentSwitch;
use crate::types::silent_with_progress_switch::SilentWithProgressSwitch;
use crate::types::urls::url::DecodedUrl;
use crate::types::version::Version;
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use const_format::formatc;
use itertools::Itertools;
use package_family_name::PackageFamilyName;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumIter, EnumString};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InstallerManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub channel: Option<String>,
    #[serde(rename = "InstallerLocale")]
    pub locale: Option<LanguageTag>,
    pub platform: Option<BTreeSet<Platform>>,
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
    pub unsupported_os_architectures: Option<BTreeSet<UnsupportedOSArchitecture>>,
    pub unsupported_arguments: Option<BTreeSet<UnsupportedArguments>>,
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

impl ManifestTrait for InstallerManifest {
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

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, EnumString, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum Platform {
    #[serde(rename = "Windows.Desktop")]
    #[strum(serialize = "Windows.Desktop")]
    WindowsDesktop,
    #[serde(rename = "Windows.Universal")]
    #[strum(serialize = "Windows.Universal")]
    WindowsUniversal,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum NestedInstallerType {
    Msix,
    Msi,
    Appx,
    Exe,
    Inno,
    Nullsoft,
    Wix,
    Burn,
    Portable,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct NestedInstallerFiles {
    pub relative_file_path: Utf8PathBuf,
    pub portable_command_alias: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Machine,
}

impl Scope {
    pub fn from_url(url: &str) -> Option<Self> {
        match url.to_ascii_lowercase() {
            url if url.contains("all-users") || url.contains("machine") => Some(Self::Machine),
            url if url.contains("user") => Some(Self::User),
            _ => None,
        }
    }

    pub fn from_install_dir(install_dir: &str) -> Option<Self> {
        const USER_INSTALL_DIRS: [&str; 2] = [RELATIVE_APP_DATA, RELATIVE_LOCAL_APP_DATA];
        const MACHINE_INSTALL_DIRS: [&str; 7] = [
            RELATIVE_PROGRAM_FILES_64,
            RELATIVE_PROGRAM_FILES_32,
            RELATIVE_COMMON_FILES_64,
            RELATIVE_COMMON_FILES_32,
            RELATIVE_PROGRAM_DATA,
            RELATIVE_WINDOWS_DIR,
            RELATIVE_SYSTEM_ROOT,
        ];

        USER_INSTALL_DIRS
            .iter()
            .any(|directory| install_dir.starts_with(directory))
            .then_some(Self::User)
            .or_else(|| {
                MACHINE_INSTALL_DIRS
                    .iter()
                    .any(|directory| install_dir.starts_with(directory))
                    .then_some(Self::Machine)
            })
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct InstallerSwitches {
    pub silent: Option<SilentSwitch>,
    pub silent_with_progress: Option<SilentWithProgressSwitch>,
    pub interactive: Option<InstallerSwitch>,
    pub install_location: Option<InstallerSwitch>,
    pub log: Option<InstallerSwitch>,
    pub upgrade: Option<InstallerSwitch>,
    pub custom: Option<CustomSwitch>,
    pub repair: Option<InstallerSwitch>,
}

impl InstallerSwitches {
    pub const fn is_any_some(&self) -> bool {
        self.silent.is_some()
            || self.silent_with_progress.is_some()
            || self.interactive.is_some()
            || self.install_location.is_some()
            || self.log.is_some()
            || self.upgrade.is_some()
            || self.custom.is_some()
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct ExpectedReturnCodes {
    pub installer_return_code: Option<NonZeroI64>,
    pub return_response: ReturnResponse,
    pub return_response_url: Option<DecodedUrl>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum ReturnResponse {
    PackageInUse,
    PackageInUseByApplication,
    InstallInProgress,
    FileInUse,
    MissingDependency,
    DiskFull,
    InsufficientMemory,
    InvalidParameter,
    NoNetwork,
    ContactSupport,
    RebootRequiredToFinish,
    RebootRequiredForInstall,
    RebootInitiated,
    CancelledByUser,
    AlreadyInstalled,
    Downgrade,
    BlockedByPolicy,
    SystemNotSupported,
    Custom,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Display,
    EnumIter,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
#[serde(rename_all = "camelCase")]
pub enum UpgradeBehavior {
    Install,
    UninstallPrevious,
    Deny,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Dependencies {
    pub windows_features: Option<BTreeSet<String>>,
    pub windows_libraries: Option<BTreeSet<String>>,
    #[serde(rename = "PackageDependencies")]
    pub package: Option<BTreeSet<PackageDependencies>>,
    #[serde(rename = "ExternalDependencies")]
    pub external: Option<BTreeSet<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct PackageDependencies {
    pub package_identifier: PackageIdentifier,
    pub minimum_version: Option<PackageVersion>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Markets {
    pub allowed_markets: Option<BTreeSet<String>>,
    pub excluded_markets: Option<BTreeSet<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum UnsupportedOSArchitecture {
    X86,
    X64,
    Arm,
    Arm64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum UnsupportedArguments {
    Log,
    Location,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct AppsAndFeaturesEntry {
    pub display_name: Option<String>,
    pub publisher: Option<String>,
    pub display_version: Option<Version>,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub installer_type: Option<InstallerType>,
}

impl AppsAndFeaturesEntry {
    pub const fn is_any_some(&self) -> bool {
        self.display_name.is_some()
            || self.publisher.is_some()
            || self.display_version.is_some()
            || self.product_code.is_some()
            || self.upgrade_code.is_some()
            || self.installer_type.is_some()
    }
}

impl AppsAndFeaturesEntry {
    pub fn deduplicate(
        &mut self,
        package_version: &PackageVersion,
        locale_manifest: &DefaultLocaleManifest,
    ) {
        if self.display_name.as_ref() == Some(&*locale_manifest.package_name) {
            self.display_name = None;
        }
        if self.publisher.as_ref() == Some(&*locale_manifest.publisher) {
            self.publisher = None;
        }
        if self.display_version.as_ref() == Some(&**package_version) {
            self.display_version = None;
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum ElevationRequirement {
    ElevationRequired,
    ElevationProhibited,
    ElevatesSelf,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct InstallationMetadata {
    pub default_install_location: Option<Utf8PathBuf>,
    pub files: Option<BTreeSet<MetadataFiles>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct MetadataFiles {
    pub relative_file_path: String,
    pub file_sha_256: Option<Sha256String>,
    pub file_type: Option<MetadataFileType>,
    pub invocation_parameter: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum MetadataFileType {
    Launch,
    Uninstall,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum RepairBehavior {
    Modify,
    Uninstaller,
    Installer,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Installer {
    #[serde(rename = "InstallerLocale")]
    pub locale: Option<LanguageTag>,
    pub platform: Option<BTreeSet<Platform>>,
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
    pub unsupported_os_architectures: Option<BTreeSet<UnsupportedOSArchitecture>>,
    pub unsupported_arguments: Option<BTreeSet<UnsupportedArguments>>,
    pub apps_and_features_entries: Option<Vec<AppsAndFeaturesEntry>>,
    pub elevation_requirement: Option<ElevationRequirement>,
    pub installation_metadata: Option<InstallationMetadata>,
    pub download_command_prohibited: Option<bool>,
    pub repair_behavior: Option<RepairBehavior>,
    pub archive_binaries_depend_on_path: Option<bool>,
}

impl Installer {
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
