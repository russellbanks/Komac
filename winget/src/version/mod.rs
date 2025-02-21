use crate::shared::language_tag::LanguageTag;
use crate::shared::manifest_type::ManifestType;
use crate::shared::manifest_version::ManifestVersion;
use crate::shared::package_identifier::PackageIdentifier;
use crate::shared::package_version::PackageVersion;
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
        self.manifest_type = ManifestType::Version;
        self.manifest_version = ManifestVersion::default();
    }
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
