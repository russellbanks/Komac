use crate::manifests::installer_manifest::Platform;
use crate::msix_family::msix;
use crate::msix_family::utils::{hash_signature, read_manifest};
use crate::types::architecture::Architecture;
use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::io::{Read, Seek};
use std::str::FromStr;
use zip::ZipArchive;

pub struct MsixBundle {
    pub signature_sha_256: String,
    pub package_family_name: String,
    pub packages: Vec<IndividualPackage>,
}

pub struct IndividualPackage {
    pub version: String,
    pub target_device_family: Platform,
    pub min_version: String,
    pub processor_architecture: Option<Architecture>,
}

const APPX_BUNDLE_MANIFEST_PATH: &str = "AppxMetadata/AppxBundleManifest.xml";

impl MsixBundle {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_bundle_manifest = read_manifest(&mut zip, APPX_BUNDLE_MANIFEST_PATH)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let bundle_manifest = from_str::<Bundle>(&appx_bundle_manifest)?;

        Ok(Self {
            signature_sha_256,
            package_family_name: get_package_family_name(
                &bundle_manifest.identity.name,
                &bundle_manifest.identity.publisher,
            ),
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
                    processor_architecture: package
                        .architecture
                        .as_deref()
                        .and_then(|architecture| Architecture::from_str(architecture).ok()),
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
    architecture: Option<String>,
    #[serde(rename = "Dependencies")]
    dependencies: msix::Dependencies,
}
