use std::collections::BTreeSet;
use std::io::{Read, Seek};

use color_eyre::eyre::Result;
use package_family_name::get_package_family_name;
use quick_xml::de::from_str;
use serde::Deserialize;
use zip::ZipArchive;

use crate::manifests::installer_manifest::Platform;
use crate::msix_family::msix;
use crate::msix_family::utils::{hash_signature, read_manifest};
use crate::types::architecture::Architecture;
use crate::types::minimum_os_version::MinimumOSVersion;

pub struct MsixBundle {
    pub signature_sha_256: String,
    pub package_family_name: String,
    pub packages: Vec<IndividualPackage>,
}

pub struct IndividualPackage {
    pub version: String,
    pub target_device_family: BTreeSet<Platform>,
    pub min_version: MinimumOSVersion,
    pub processor_architecture: Architecture,
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
                .filter(|package| package.r#type == PackageType::Application)
                .map(|package| IndividualPackage {
                    version: package.version,
                    target_device_family: package
                        .dependencies
                        .target_device_family
                        .iter()
                        .map(|target_device_family| target_device_family.name)
                        .collect(),
                    min_version: package
                        .dependencies
                        .target_device_family
                        .into_iter()
                        .map(|target_device_family| target_device_family.min_version)
                        .min()
                        .unwrap(),
                    processor_architecture: package.architecture,
                })
                .collect(),
        })
    }
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-bundle>
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Bundle {
    identity: Identity,
    packages: Packages,
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-identity>
#[derive(Deserialize)]
struct Identity {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Publisher")]
    publisher: String,
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-packages>
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Packages {
    package: Vec<Package>,
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-package>
#[derive(Deserialize)]
struct Package {
    #[serde(default, rename = "@Architecture")]
    architecture: Architecture,
    #[serde(default, rename = "@Type")]
    r#type: PackageType,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "Dependencies")]
    dependencies: msix::Dependencies,
}

/// <https://learn.microsoft.com/en-gb/uwp/schemas/bundlemanifestschema/element-package#attributes>
#[derive(Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum PackageType {
    Application,
    #[default]
    Resource,
}
