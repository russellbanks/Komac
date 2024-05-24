use std::collections::BTreeSet;
use std::io::{Read, Seek};

use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use zip::ZipArchive;

use crate::manifests::installer_manifest::Platform;
use crate::msix_family::utils::{hash_signature, read_manifest};
use crate::types::architecture::Architecture;
use crate::types::minimum_os_version::MinimumOSVersion;

pub struct Msix {
    pub display_name: String,
    pub publisher_display_name: String,
    pub version: String,
    pub signature_sha_256: String,
    pub package_family_name: String,
    pub target_device_family: BTreeSet<Platform>,
    pub min_version: MinimumOSVersion,
    pub processor_architecture: Architecture,
}

const APPX_MANIFEST_XML: &str = "AppxManifest.xml";
pub const APPX_SIGNATURE_P7X: &str = "AppxSignature.p7x";

impl Msix {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_manifest = read_manifest(&mut zip, APPX_MANIFEST_XML)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let manifest = from_str::<Package>(&appx_manifest)?;

        Ok(Self {
            display_name: manifest.properties.display_name,
            publisher_display_name: manifest.properties.publisher_display_name,
            version: manifest.identity.version,
            signature_sha_256,
            package_family_name: get_package_family_name(
                &manifest.identity.name,
                &manifest.identity.publisher,
            ),
            target_device_family: manifest
                .dependencies
                .target_device_family
                .iter()
                .map(|target_device_family| target_device_family.name)
                .collect(),
            min_version: manifest
                .dependencies
                .target_device_family
                .into_iter()
                .map(|target_device_family| target_device_family.min_version)
                .min()
                .unwrap(),
            processor_architecture: manifest.identity.processor_architecture,
        })
    }
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-package>
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Package {
    identity: Identity,
    properties: Properties,
    dependencies: Dependencies,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-identity>
#[derive(Deserialize)]
struct Identity {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(default, rename = "@ProcessorArchitecture")]
    processor_architecture: Architecture,
    #[serde(rename = "@Publisher")]
    publisher: String,
    #[serde(rename = "@Version")]
    version: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-properties>
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Properties {
    display_name: String,
    publisher_display_name: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-dependencies>
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Dependencies {
    pub target_device_family: BTreeSet<TargetDeviceFamily>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-targetdevicefamily>
#[derive(Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TargetDeviceFamily {
    #[serde(rename = "@Name")]
    pub name: Platform,
    #[serde(rename = "@MinVersion")]
    pub min_version: MinimumOSVersion,
}
