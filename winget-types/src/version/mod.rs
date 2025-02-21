use const_format::formatc;
use serde::{Deserialize, Serialize};

use crate::{
    shared::{LanguageTag, ManifestType, ManifestVersion, PackageIdentifier, PackageVersion},
    traits::Manifest,
};

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
        self.manifest_type = ManifestType::Version;
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

impl Default for VersionManifest {
    fn default() -> Self {
        Self {
            package_identifier: PackageIdentifier::default(),
            package_version: PackageVersion::default(),
            default_locale: LanguageTag::default(),
            manifest_type: ManifestType::Version,
            manifest_version: ManifestVersion::default(),
        }
    }
}
