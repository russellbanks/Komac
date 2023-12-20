use crate::manifests::installer_manifest::{Architecture, Platform};
use crate::msix_family::msix_utils::get_manifest_and_signature;
use crate::types::minimum_os_version::MinimumOSVersion;
use async_zip::tokio::read::seek::ZipFileReader;
use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use tokio::fs::File;

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

impl Msix {
    pub async fn new(file: &mut File) -> Result<Msix> {
        let zip = ZipFileReader::with_tokio(file).await?;

        let (appx_manifest, appx_signature) =
            get_manifest_and_signature(zip, APPX_MANIFEST_XML).await?;

        let signature_hash = Sha256::digest(appx_signature);
        let signature_sha_256 = base16ct::upper::encode_string(&signature_hash);

        let manifest: Package = from_str(&appx_manifest)?;

        let package_family_name =
            get_package_family_name(&manifest.identity.name, &manifest.identity.publisher);

        Ok(Msix {
            display_name: manifest.properties.display_name,
            publisher_display_name: manifest.properties.publisher_display_name,
            version: manifest.identity.version,
            signature_sha_256,
            package_family_name,
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
