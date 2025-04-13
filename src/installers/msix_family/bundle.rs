use std::{
    io,
    io::{Cursor, Read, Seek},
};

use color_eyre::eyre::Result;
use itertools::Itertools;
use memmap2::Mmap;
use quick_xml::de::from_str;
use serde::Deserialize;
use winget_types::installer::{Installer, PackageFamilyName};
use zip::ZipArchive;

use crate::installers::msix_family::{
    Msix,
    utils::{hash_signature, read_manifest},
};

pub struct MsixBundle {
    pub installers: Vec<Installer>,
}

const APPX_BUNDLE_MANIFEST_PATH: &str = "AppxMetadata/AppxBundleManifest.xml";

impl MsixBundle {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_bundle_manifest = read_manifest(&mut zip, APPX_BUNDLE_MANIFEST_PATH)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let bundle_manifest = from_str::<Bundle>(&appx_bundle_manifest)?;

        let package_family_name = PackageFamilyName::new(
            &bundle_manifest.identity.name,
            &bundle_manifest.identity.publisher,
        );

        Ok(Self {
            installers: bundle_manifest
                .packages
                .package
                .into_iter()
                .filter(|package| package.r#type == PackageType::Application)
                .map(|package| {
                    let mut embedded_msix = zip.by_name(&package.file_name)?;
                    let mut temp_file = tempfile::tempfile()?;
                    io::copy(&mut embedded_msix, &mut temp_file)?;
                    let map = unsafe { Mmap::map(&temp_file) }?;
                    Msix::new(Cursor::new(map.as_ref()))
                })
                .map_ok(|msix| Installer {
                    signature_sha_256: Some(signature_sha_256.clone()),
                    package_family_name: Some(package_family_name.clone()),
                    ..msix.installer
                })
                .collect::<Result<Vec<_>>>()?,
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
    #[serde(default, rename = "@Type")]
    r#type: PackageType,
    #[serde(rename = "@FileName")]
    file_name: String,
}

/// <https://learn.microsoft.com/en-gb/uwp/schemas/bundlemanifestschema/element-package#attributes>
#[derive(Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum PackageType {
    Application,
    #[default]
    Resource,
}
