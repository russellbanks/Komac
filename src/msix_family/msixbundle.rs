use crate::installer_manifest::{Architecture, Platform};
use crate::msix_family::msix;
use crate::msix_family::msix_utils::get_manifest_and_signature;
use async_zip::tokio::read::seek::ZipFileReader;
use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use tokio::fs::File;

pub struct MsixBundle {
    pub signature_sha_256: String,
    pub package_family_name: String,
    pub packages: Vec<IndividualPackage>,
}

pub struct IndividualPackage {
    pub version: String,
    pub target_device_family: Platform,
    pub min_version: String,
    pub processor_architecture: Architecture,
}

const APPX_BUNDLE_MANIFEST_PATH: &str = "AppxMetadata/AppxBundleManifest.xml";

impl MsixBundle {
    pub async fn new(file: &mut File) -> Result<MsixBundle> {
        let zip = ZipFileReader::with_tokio(file).await?;

        let (appx_bundle_manifest, appx_signature) =
            get_manifest_and_signature(zip, APPX_BUNDLE_MANIFEST_PATH).await?;

        let signature_hash = Sha256::digest(appx_signature);
        let signature_sha_256 = base16ct::upper::encode_string(&signature_hash);

        let bundle_manifest: Bundle = from_str(&appx_bundle_manifest)?;

        let package_family_name = get_package_family_name(
            &bundle_manifest.identity.name,
            &bundle_manifest.identity.publisher,
        );

        Ok(MsixBundle {
            signature_sha_256,
            package_family_name,
            packages: bundle_manifest
                .packages
                .package
                .into_iter()
                .map(|package| IndividualPackage {
                    version: package.version,
                    target_device_family: Platform::from_str(
                        &package.dependencies.target_device_family.name,
                    )
                    .unwrap(),
                    min_version: package.dependencies.target_device_family.min_version,
                    processor_architecture: Architecture::from_str(&package.architecture).unwrap(),
                })
                .collect(),
        })
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Bundle {
    identity: Identity,
    packages: Packages,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Identity {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Publisher")]
    publisher: String,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Packages {
    package: Vec<Package>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Package {
    #[serde(rename = "@Type")]
    r#type: String,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "@Architecture")]
    architecture: String,
    #[serde(rename = "Dependencies")]
    dependencies: msix::Dependencies,
}
