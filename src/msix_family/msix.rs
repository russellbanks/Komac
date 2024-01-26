use crate::manifests::installer_manifest::Platform;
use crate::msix_family::utils::{hash_signature, read_manifest};
use crate::types::architecture::Architecture;
use crate::types::minimum_os_version::MinimumOSVersion;
use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::io::{Read, Seek};
use std::str::FromStr;
use zip::ZipArchive;

pub struct Msix {
    pub display_name: String,
    pub publisher_display_name: String,
    pub version: String,
    pub signature_sha_256: String,
    pub package_family_name: String,
    pub target_device_family: Platform,
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
            target_device_family: Platform::from_str(
                &manifest.dependencies.target_device_family.name,
            )?,
            min_version: MinimumOSVersion::new(
                manifest.dependencies.target_device_family.min_version,
            )?,
            processor_architecture: Architecture::from_str(
                &manifest.identity.processor_architecture,
            )?,
        })
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Package {
    identity: Identity,
    properties: Properties,
    dependencies: Dependencies,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Identity {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "@Publisher")]
    publisher: String,
    #[serde(rename = "@ProcessorArchitecture")]
    processor_architecture: String,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Properties {
    display_name: String,
    publisher_display_name: String,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub(super) struct Dependencies {
    pub target_device_family: TargetDeviceFamily,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub(super) struct TargetDeviceFamily {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@MinVersion")]
    pub min_version: String,
}
