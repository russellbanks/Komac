use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::mem;

use camino::Utf8Path;
use chrono::NaiveDate;
use color_eyre::eyre::Result;
use yara_x::mods::pe::{Resource, ResourceType};
use yara_x::mods::PE;

use crate::manifests::installer_manifest::{Platform, Scope};
use crate::msi::Msi;
use crate::msix_family::msix::Msix;
use crate::msix_family::msixbundle::MsixBundle;
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use crate::zip::Zip;

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";

pub struct FileAnalyser<'a> {
    pub platform: Option<BTreeSet<Platform>>,
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Option<Architecture>,
    pub installer_type: InstallerType,
    pub scope: Option<Scope>,
    pub installer_sha_256: String,
    pub signature_sha_256: Option<String>,
    pub package_family_name: Option<String>,
    pub product_code: Option<String>,
    pub product_language: Option<LanguageTag>,
    pub last_modified: Option<NaiveDate>,
    pub file_name: Cow<'a, str>,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub msi: Option<Msi>,
    pub zip: Option<Zip>,
}

impl<'a> FileAnalyser<'a> {
    pub fn new(data: &[u8], file_name: Cow<'a, str>) -> Result<Self> {
        let extension = Utf8Path::new(file_name.as_ref())
            .extension()
            .unwrap_or_default()
            .to_lowercase();
        let mut msi = match extension.as_str() {
            MSI => Some(Msi::new(Cursor::new(data))?),
            _ => None,
        };
        let mut installer_type = None;
        let mut pe_arch = None;
        let pe = yara_x::mods::invoke::<PE>(data);
        if let Some(ref pe) = pe {
            pe_arch = Some(Architecture::get_from_exe(pe)?);
            installer_type = Some(InstallerType::get(
                data,
                Some(pe),
                &extension,
                msi.as_ref(),
            )?);
            if let Some(msi_resource) = get_msi_resource(pe) {
                installer_type = Some(InstallerType::Burn);
                msi = Some(extract_msi(data, msi_resource)?);
            }
        }
        let mut msix = match extension.as_str() {
            MSIX | APPX => Some(Msix::new(Cursor::new(data))?),
            _ => None,
        };
        let mut msix_bundle = match extension.as_str() {
            MSIX_BUNDLE | APPX_BUNDLE => Some(MsixBundle::new(Cursor::new(data))?),
            _ => None,
        };
        let mut zip = match extension.as_str() {
            ZIP => Some(Zip::new(Cursor::new(data))?),
            _ => None,
        };
        if installer_type.is_none() {
            installer_type = Some(InstallerType::get(
                data,
                pe.as_ref(),
                &extension,
                msi.as_ref(),
            )?);
        }
        Ok(Self {
            platform: msix
                .as_ref()
                .map(|msix| BTreeSet::from([msix.target_device_family])),
            minimum_os_version: msix.as_mut().map(|msix| mem::take(&mut msix.min_version)),
            architecture: msi
                .as_ref()
                .map(|msi| msi.architecture)
                .or_else(|| msix.as_ref().map(|msix| msix.processor_architecture))
                .or_else(|| {
                    msix_bundle.as_ref().and_then(|bundle| {
                        bundle
                            .packages
                            .iter()
                            .find_map(|package| package.processor_architecture)
                    })
                })
                .or(pe_arch)
                .or_else(|| {
                    zip.as_mut()
                        .and_then(|zip| mem::take(&mut zip.architecture))
                }),
            installer_type: installer_type.unwrap(),
            scope: msi.as_ref().and_then(|msi| msi.all_users),
            installer_sha_256: String::new(),
            signature_sha_256: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.signature_sha_256))
                .or_else(|| {
                    msix_bundle
                        .as_mut()
                        .map(|msix_bundle| mem::take(&mut msix_bundle.signature_sha_256))
                }),
            package_family_name: msix
                .map(|msix| msix.package_family_name)
                .or_else(|| msix_bundle.map(|msix_bundle| msix_bundle.package_family_name)),
            product_code: msi.as_mut().map(|msi| mem::take(&mut msi.product_code)),
            product_language: msi.as_mut().map(|msi| mem::take(&mut msi.product_language)),
            last_modified: None,
            file_name,
            copyright: pe
                .as_ref()
                .and_then(|pe| Copyright::get_from_exe(&pe.version_info)),
            package_name: pe
                .as_ref()
                .and_then(|pe| PackageName::get_from_exe(&pe.version_info)),
            publisher: pe
                .as_ref()
                .and_then(|pe| Publisher::get_from_exe(&pe.version_info)),
            msi,
            zip,
        })
    }
}

fn get_msi_resource(pe: &Box<PE>) -> Option<&Resource> {
    const MSI: &[u8] = b"M\0S\0I\0";

    pe.resources
        .iter()
        .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
        .find(|resource| resource.name_string() == MSI)
}

pub fn extract_msi(data: &[u8], msi_resource: &Resource) -> Result<Msi> {
    let offset = msi_resource.offset() as usize;
    let data = &data[offset..offset + msi_resource.length() as usize];
    let msi = Msi::new(Cursor::new(data))?;
    Ok(msi)
}
