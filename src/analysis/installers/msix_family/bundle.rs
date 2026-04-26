use std::{
    borrow::Cow,
    io,
    io::{BufReader, Read, Seek},
};

use color_eyre::Result;
use percent_encoding::percent_decode_str;
use quick_xml::de::from_str;
use serde::Deserialize;
use winget_types::{
    Sha256String,
    installer::{Installer, PackageFamilyName},
};
use zip::ZipArchive;

use super::{
    Msix,
    utils::{hash_signature, read_manifest},
};
use crate::analysis::Installers;

pub struct MsixBundle {
    pub signature_sha_256: Sha256String,
    pub package_family_name: PackageFamilyName<'static>,
    pub msix_files: Vec<Msix>,
}

const APPX_BUNDLE_MANIFEST_PATH: &str = "AppxMetadata/AppxBundleManifest.xml";

impl MsixBundle {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_bundle_manifest = read_manifest(&mut zip, APPX_BUNDLE_MANIFEST_PATH)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let bundle_manifest = from_str::<Bundle>(&appx_bundle_manifest)?;

        let package_family_name = PackageFamilyName::new(
            bundle_manifest.identity.name.clone(),
            &bundle_manifest.identity.publisher,
        );

        Ok(Self {
            msix_files: bundle_manifest
                .packages
                .package
                .iter()
                .filter(|package| package.is_application())
                .map(|package| {
                    // Find file by package file name, comparing by decoded file names
                    let file_name = zip
                        .file_names()
                        .find(|file_name| {
                            percent_decode_str(file_name)
                                .eq(percent_decode_str(package.file_name()))
                        })
                        .map(|file_name| Cow::Owned(file_name.to_owned()))
                        .unwrap_or(Cow::Borrowed(package.file_name()));

                    let mut embedded_msix = zip.by_name(&file_name)?;
                    let mut temp_file = tempfile::tempfile()?;
                    io::copy(&mut embedded_msix, &mut temp_file)?;
                    Msix::new(BufReader::new(temp_file))
                })
                .collect::<Result<Vec<_>>>()?,
            signature_sha_256,
            package_family_name,
        })
    }
}

impl Installers for MsixBundle {
    fn installers(&self) -> Vec<Installer> {
        self.msix_files
            .iter()
            .map(|msix| Installer {
                signature_sha_256: Some(self.signature_sha_256.clone()),
                package_family_name: Some(self.package_family_name.clone()),
                ..msix.installers().swap_remove(0)
            })
            .collect::<Vec<_>>()
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

impl Package {
    #[inline]
    pub const fn is_application(&self) -> bool {
        self.r#type.is_application()
    }

    #[expect(unused)]
    #[inline]
    pub const fn is_resource(&self) -> bool {
        self.r#type.is_resource()
    }

    #[inline]
    pub const fn file_name(&self) -> &str {
        self.file_name.as_str()
    }
}

/// <https://learn.microsoft.com/en-gb/uwp/schemas/bundlemanifestschema/element-package#attributes>
#[derive(Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum PackageType {
    Application,
    #[default]
    Resource,
}

impl PackageType {
    #[inline]
    pub const fn is_application(&self) -> bool {
        matches!(self, Self::Application)
    }

    #[inline]
    pub const fn is_resource(&self) -> bool {
        matches!(self, Self::Resource)
    }
}
