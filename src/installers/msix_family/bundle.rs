use crate::installers::msix_family::utils::{hash_signature, read_manifest};
use crate::installers::msix_family::Msix;
use crate::installers::traits::InstallSpec;
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;
use crate::types::sha_256::Sha256String;
use color_eyre::eyre::Result;
use memmap2::Mmap;
use package_family_name::PackageFamilyName;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::io;
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

pub struct MsixBundle {
    pub signature_sha_256: Sha256String,
    pub package_family_name: PackageFamilyName,
    pub packages: Vec<Msix>,
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
            package_family_name: PackageFamilyName::new(
                &bundle_manifest.identity.name,
                &bundle_manifest.identity.publisher,
            ),
            packages: bundle_manifest
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

impl InstallSpec for MsixBundle {
    fn r#type(&self) -> InstallerType {
        if self.packages.first().is_some_and(|msix| msix.is_appx) {
            InstallerType::Appx
        } else {
            InstallerType::Msix
        }
    }

    fn architecture(&self) -> Option<Architecture> {
        self.packages
            .first()
            .map(|msix| msix.processor_architecture)
    }

    fn signature_sha_256(&self) -> Option<Sha256String> {
        Some(self.signature_sha_256.clone())
    }

    fn package_family_name(&self) -> Option<PackageFamilyName> {
        Some(self.package_family_name.clone())
    }
}
