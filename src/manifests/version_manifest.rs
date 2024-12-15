use crate::manifests::Manifest;
use crate::types::language_tag::LanguageTag;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use const_format::formatc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VersionManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub default_locale: LanguageTag,
    pub manifest_type: ManifestType,
    #[serde(default)]
    pub manifest_version: ManifestVersion,
}

impl VersionManifest {
    pub fn update(&mut self, package_version: &PackageVersion) {
        self.package_version.clone_from(package_version);
        self.manifest_type = Self::TYPE;
        self.manifest_version = ManifestVersion::default();
    }
}

impl Manifest for VersionManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.version.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::Version;
}
