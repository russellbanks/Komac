use crate::types::language_tag::LanguageTag;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
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
