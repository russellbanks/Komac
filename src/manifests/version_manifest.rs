use crate::manifests::ManifestTrait;
use const_format::formatc;
use winget::shared::manifest_type::ManifestType;
use winget::shared::manifest_version::ManifestVersion;
use winget::version::VersionManifest;

impl ManifestTrait for VersionManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.version.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::Version;
}
