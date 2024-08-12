use std::collections::BTreeSet;
use std::io::Cursor;
use std::mem;

use crate::installers::inno::inno::InnoFile;
use crate::installers::msi::Msi;
use crate::installers::msix_family::msix::Msix;
use crate::installers::msix_family::msixbundle::MsixBundle;
use crate::installers::zip::Zip;
use crate::manifests::installer_manifest::{Platform, Scope};
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::file_extension::FileExtension;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use crate::types::sha_256::Sha256String;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDate;
use color_eyre::eyre::Result;
use memmap2::Mmap;
use package_family_name::PackageFamilyName;
use yara_x::mods::pe::{Resource, ResourceType};
use yara_x::mods::PE;

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";

pub struct FileAnalyser<'data> {
    pub platform: Option<BTreeSet<Platform>>,
    pub minimum_os_version: Option<MinimumOSVersion>,
    pub architecture: Option<Architecture>,
    pub installer_type: InstallerType,
    pub scope: Option<Scope>,
    pub installer_sha_256: Sha256String,
    pub signature_sha_256: Option<Sha256String>,
    pub package_family_name: Option<PackageFamilyName>,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub capabilities: Option<BTreeSet<String>>,
    pub restricted_capabilities: Option<BTreeSet<String>>,
    pub file_extensions: Option<BTreeSet<FileExtension>>,
    pub product_language: Option<LanguageTag>,
    pub last_modified: Option<NaiveDate>,
    pub display_name: Option<String>,
    pub display_publisher: Option<String>,
    pub display_version: Option<String>,
    pub file_name: String,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub default_install_location: Option<Utf8PathBuf>,
    pub zip: Option<Zip<Cursor<&'data [u8]>>>,
}

impl<'data> FileAnalyser<'data> {
    pub fn new(data: &'data Mmap, file_name: &str) -> Result<Self> {
        let extension = Utf8Path::new(file_name)
            .extension()
            .unwrap_or_default()
            .to_lowercase();
        let mut msi = None;
        let mut msix = None;
        let mut msix_bundle = None;
        let mut zip = None;
        match extension.as_str() {
            MSI => msi = Some(Msi::new(Cursor::new(data.as_ref()))?),
            MSIX | APPX => msix = Some(Msix::new(Cursor::new(data.as_ref()))?),
            MSIX_BUNDLE | APPX_BUNDLE => {
                msix_bundle = Some(MsixBundle::new(Cursor::new(data.as_ref()))?);
            }
            ZIP => zip = Some(Zip::new(Cursor::new(data.as_ref()))?),
            _ => {}
        }
        let mut inno = None;
        let mut installer_type = None;
        let mut pe_arch = None;
        let pe = yara_x::mods::invoke::<PE>(data.as_ref());
        if let Some(ref pe) = pe {
            pe_arch = Some(Architecture::get_from_exe(pe)?);
            installer_type = Some(InstallerType::get(
                data.as_ref(),
                Some(pe),
                &extension,
                msi.as_ref(),
            )?);
            if installer_type == Some(InstallerType::Inno) {
                inno = InnoFile::new(data.as_ref(), pe).ok()
            }
            if let Some(msi_resource) = get_msi_resource(pe) {
                installer_type = Some(InstallerType::Burn);
                msi = Some(extract_msi(data.as_ref(), msi_resource)?);
            }
        }
        if installer_type.is_none() {
            installer_type = Some(InstallerType::get(
                data.as_ref(),
                pe.as_deref(),
                &extension,
                msi.as_ref(),
            )?);
        }
        Ok(Self {
            platform: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.target_device_family)),
            minimum_os_version: msix.as_mut().map(|msix| mem::take(&mut msix.min_version)),
            architecture: msi
                .as_ref()
                .map(|msi| msi.architecture)
                .or_else(|| msix.as_ref().map(|msix| msix.processor_architecture))
                .or_else(|| {
                    msix_bundle.as_ref().and_then(|bundle| {
                        bundle
                            .packages
                            .first()
                            .map(|package| package.processor_architecture)
                    })
                })
                .or(pe_arch)
                .or_else(|| {
                    zip.as_mut()
                        .and_then(|zip| mem::take(&mut zip.architecture))
                }),
            installer_type: installer_type.unwrap(),
            scope: msi.as_ref().and_then(|msi| msi.all_users),
            installer_sha_256: Sha256String::default(),
            signature_sha_256: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.signature_sha_256))
                .or_else(|| {
                    msix_bundle
                        .as_mut()
                        .map(|msix_bundle| mem::take(&mut msix_bundle.signature_sha_256))
                }),
            package_family_name: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.package_family_name))
                .or_else(|| msix_bundle.map(|msix_bundle| msix_bundle.package_family_name)),
            product_code: msi
                .as_mut()
                .map(|msi| mem::take(&mut msi.product_code))
                .or_else(|| {
                    inno.as_mut()
                        .and_then(|inno| mem::take(&mut inno.product_code))
                }),
            upgrade_code: msi.as_mut().map(|msi| mem::take(&mut msi.upgrade_code)),
            capabilities: msix
                .as_mut()
                .and_then(|msix| mem::take(&mut msix.capabilities)),
            restricted_capabilities: msix
                .as_mut()
                .and_then(|msix| mem::take(&mut msix.restricted_capabilities)),
            file_extensions: msix
                .as_mut()
                .and_then(|msix| mem::take(&mut msix.file_extensions)),
            product_language: msi.as_mut().map(|msi| mem::take(&mut msi.product_language)),
            last_modified: None,
            display_name: msi
                .as_mut()
                .map(|msi| mem::take(&mut msi.product_name))
                .or_else(|| {
                    inno.as_mut()
                        .and_then(|inno| mem::take(&mut inno.uninstall_name))
                }),
            display_publisher: msi
                .as_mut()
                .map(|msi| mem::take(&mut msi.manufacturer))
                .or_else(|| {
                    inno.as_mut()
                        .and_then(|inno| mem::take(&mut inno.app_publisher))
                }),
            display_version: msi
                .as_mut()
                .map(|msi| mem::take(&mut msi.product_version))
                .or_else(|| {
                    inno.as_mut()
                        .and_then(|inno| mem::take(&mut inno.app_version))
                }),
            file_name: String::new(),
            copyright: pe
                .as_ref()
                .and_then(|pe| Copyright::get_from_exe(&pe.version_info)),
            package_name: pe
                .as_ref()
                .and_then(|pe| PackageName::get_from_exe(&pe.version_info)),
            publisher: pe
                .as_ref()
                .and_then(|pe| Publisher::get_from_exe(&pe.version_info)),
            default_install_location: msi
                .and_then(|msi| msi.install_location)
                .or_else(|| msix.map(|msix| msix.install_location)),
            zip,
        })
    }
}

fn get_msi_resource(pe: &PE) -> Option<&Resource> {
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
