pub mod abs_to_u32;

use crate::manifests::installer_manifest::{
    ElevationRequirement, Platform, Scope, UnsupportedOSArchitecture,
};
use crate::types::architecture::Architecture;
use crate::types::file_extension::FileExtension;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::sha_256::Sha256String;
use camino::Utf8PathBuf;
use package_family_name::PackageFamilyName;
use std::collections::BTreeSet;
use versions::Versioning;

pub trait InstallSpec {
    fn r#type(&self) -> InstallerType;

    fn architecture(&self) -> Option<Architecture> {
        None
    }

    fn display_name(&self) -> Option<String> {
        None
    }

    fn display_publisher(&self) -> Option<String> {
        None
    }

    fn display_version(&self) -> Option<Versioning> {
        None
    }

    fn product_code(&self) -> Option<String> {
        None
    }

    fn locale(&self) -> Option<LanguageTag> {
        None
    }

    fn platform(&self) -> Option<BTreeSet<Platform>> {
        None
    }

    fn scope(&self) -> Option<Scope> {
        None
    }

    fn unsupported_os_architectures(&self) -> Option<BTreeSet<UnsupportedOSArchitecture>> {
        None
    }

    fn elevation_requirement(&self) -> Option<ElevationRequirement> {
        None
    }

    fn install_location(&self) -> Option<Utf8PathBuf> {
        None
    }

    fn min_version(&self) -> Option<MinimumOSVersion> {
        None
    }

    fn signature_sha_256(&self) -> Option<Sha256String> {
        None
    }

    fn file_extensions(&self) -> Option<BTreeSet<FileExtension>> {
        None
    }

    fn package_family_name(&self) -> Option<PackageFamilyName> {
        None
    }

    fn capabilities(&self) -> Option<BTreeSet<String>> {
        None
    }

    fn restricted_capabilities(&self) -> Option<BTreeSet<String>> {
        None
    }

    fn upgrade_code(&self) -> Option<String> {
        None
    }
}
