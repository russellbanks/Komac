use crate::types::architecture::Architecture;
use crate::types::command::Command;
use crate::types::custom_switch::CustomSwitch;
use crate::types::file_extension::FileExtension;
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
use crate::types::silent_switch::SilentSwitch;
use crate::types::silent_with_progress_switch::SilentWithProgressSwitch;
use crate::types::urls::url::Url;
use camino::Utf8PathBuf;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeSet;
use std::num::NonZeroI64;
use strum::{Display, EnumIter, EnumString};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InstallerManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub channel: Option<String>,
    pub installer_locale: Option<LanguageTag>,
    pub platform: Option<BTreeSet<Platform>>,
    #[serde(rename = "MinimumOSVersion")]
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub installer_type: Option<InstallerType>,
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub scope: Option<Scope>,
    pub install_modes: Option<BTreeSet<InstallModes>>,
    pub installer_switches: Option<InstallerSwitches>,
    pub installer_success_codes: Option<BTreeSet<InstallerSuccessCode>>,
    pub expected_return_codes: Option<BTreeSet<ExpectedReturnCodes>>,
    pub upgrade_behavior: Option<UpgradeBehavior>,
    pub commands: Option<BTreeSet<Command>>,
    pub protocols: Option<BTreeSet<Protocol>>,
    pub file_extensions: Option<BTreeSet<FileExtension>>,
    pub dependencies: Option<Dependencies>,
    pub package_family_name: Option<String>,
    pub product_code: Option<String>,
    pub capabilities: Option<BTreeSet<String>>,
    pub restricted_capabilities: Option<BTreeSet<String>>,
    pub markets: Option<Markets>,
    pub installer_aborts_terminal: Option<bool>,
    pub release_date: Option<NaiveDate>,
    pub installer_location_required: Option<bool>,
    pub require_explicit_upgrade: Option<bool>,
    pub display_install_warnings: Option<bool>,
    #[serde(rename = "UnsupportedOSArchitectures")]
    pub unsupported_os_architectures: Option<BTreeSet<UnsupportedOSArchitectures>>,
    pub unsupported_arguments: Option<BTreeSet<UnsupportedArguments>>,
    pub apps_and_features_entries: Option<BTreeSet<AppsAndFeaturesEntry>>,
    pub elevation_requirement: Option<ElevationRequirement>,
    pub installation_metadata: Option<InstallationMetadata>,
    pub download_command_prohibited: Option<bool>,
    pub installers: BTreeSet<Installer>,
    pub manifest_type: ManifestType,
    #[serde(default)]
    pub manifest_version: ManifestVersion,
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
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
    pub fn find_from_url(url: &str) -> Option<Self> {
        match url.to_ascii_lowercase() {
            url if url.contains("all-users") || url.contains("machine") => Some(Self::Machine),
            url if url.contains("user") => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Display, EnumIter, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[serde(rename_all = "camelCase")]
pub enum InstallModes {
    Interactive,
    Silent,
    SilentWithProgress,
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
    pub return_response_url: Option<Url>,
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

impl UpgradeBehavior {
    pub const fn get(installer_type: InstallerType) -> Option<Self> {
        match installer_type {
            InstallerType::Msix | InstallerType::Appx => Some(Self::Install),
            _ => None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Dependencies {
    pub windows_features: Option<BTreeSet<String>>,
    pub windows_libraries: Option<BTreeSet<String>>,
    pub package_dependencies: Option<BTreeSet<PackageDependencies>>,
    pub external_dependencies: Option<BTreeSet<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct PackageDependencies {
    pub package_identifier: String,
    pub minimum_version: Option<String>,
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
pub enum UnsupportedOSArchitectures {
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
    pub display_version: Option<String>,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub installer_type: Option<InstallerType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum ElevationRequirement {
    ElevationRequired,
    ElevationProhibited,
    ElevatesSelf,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct InstallationMetadata {
    pub default_install_location: Option<String>,
    pub files: Option<BTreeSet<MetadataFiles>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct MetadataFiles {
    pub relative_file_path: String,
    pub file_sha_256: Option<String>,
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Installer {
    pub installer_locale: Option<LanguageTag>,
    pub platform: Option<BTreeSet<Platform>>,
    #[serde(rename = "MinimumOSVersion")]
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Architecture,
    pub installer_type: Option<InstallerType>,
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub scope: Option<Scope>,
    pub installer_url: Url,
    pub installer_sha_256: String,
    pub signature_sha_256: Option<String>,
    pub install_modes: Option<BTreeSet<InstallModes>>,
    pub installer_switches: Option<InstallerSwitches>,
    pub installer_success_codes: Option<BTreeSet<InstallerSuccessCode>>,
    pub expected_return_codes: Option<BTreeSet<ExpectedReturnCodes>>,
    pub upgrade_behavior: Option<UpgradeBehavior>,
    pub commands: Option<BTreeSet<Command>>,
    pub protocols: Option<BTreeSet<Protocol>>,
    pub file_extensions: Option<BTreeSet<FileExtension>>,
    pub dependencies: Option<Dependencies>,
    pub package_family_name: Option<String>,
    pub product_code: Option<String>,
    pub capabilities: Option<BTreeSet<String>>,
    pub restricted_capabilities: Option<BTreeSet<String>>,
    pub markets: Option<Markets>,
    pub installer_aborts_terminal: Option<bool>,
    pub release_date: Option<NaiveDate>,
    pub installer_location_required: Option<bool>,
    pub require_explicit_upgrade: Option<bool>,
    pub display_install_warnings: Option<bool>,
    #[serde(rename = "UnsupportedOSArchitectures")]
    pub unsupported_os_architectures: Option<BTreeSet<UnsupportedOSArchitectures>>,
    pub unsupported_arguments: Option<BTreeSet<UnsupportedArguments>>,
    pub apps_and_features_entries: Option<BTreeSet<AppsAndFeaturesEntry>>,
    pub elevation_requirement: Option<ElevationRequirement>,
    pub installation_metadata: Option<InstallationMetadata>,
    pub download_command_prohibited: Option<bool>,
}
