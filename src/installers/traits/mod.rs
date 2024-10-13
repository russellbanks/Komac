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

pub trait InstallSpec {
    fn r#type(&self) -> InstallerType;

    fn architecture(&mut self) -> Option<Architecture> {
        None
    }

    fn display_name(&mut self) -> Option<String> {
        None
    }

    fn display_publisher(&mut self) -> Option<String> {
        None
    }

    fn display_version(&mut self) -> Option<String> {
        None
    }

    fn product_code(&mut self) -> Option<String> {
        None
    }

    fn locale(&mut self) -> Option<LanguageTag> {
        None
    }

    fn platform(&mut self) -> Option<BTreeSet<Platform>> {
        None
    }

    fn scope(&self) -> Option<Scope> {
        None
    }

    fn unsupported_os_architectures(&mut self) -> Option<BTreeSet<UnsupportedOSArchitecture>> {
        None
    }

    fn elevation_requirement(&mut self) -> Option<ElevationRequirement> {
        None
    }

    fn install_location(&mut self) -> Option<Utf8PathBuf> {
        None
    }

    fn min_version(&self) -> Option<MinimumOSVersion> {
        None
    }

    fn signature_sha_256(&mut self) -> Option<Sha256String> {
        None
    }

    fn file_extensions(&mut self) -> Option<BTreeSet<FileExtension>> {
        None
    }

    fn package_family_name(&mut self) -> Option<PackageFamilyName> {
        None
    }

    fn capabilities(&mut self) -> Option<BTreeSet<String>> {
        None
    }

    fn restricted_capabilities(&mut self) -> Option<BTreeSet<String>> {
        None
    }

    fn upgrade_code(&mut self) -> Option<String> {
        None
    }
}
