use std::io::Cursor;
use std::mem;

use crate::installers::inno::InnoFile;
use crate::installers::msi::Msi;
use crate::installers::msix_family::bundle::MsixBundle;
use crate::installers::msix_family::Msix;
use crate::installers::nsis::Nsis;
use crate::installers::zip::Zip;
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, InstallationMetadata, Installer, UpgradeBehavior,
};
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::installer_type::InstallerType;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use crate::types::sha_256::Sha256String;
use crate::types::urls::url::DecodedUrl;
use camino::Utf8Path;
use color_eyre::eyre::Result;
use memmap2::Mmap;
use versions::Versioning;
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
    pub file_name: String,
    pub copyright: Option<Copyright>,
    pub package_name: Option<PackageName>,
    pub publisher: Option<Publisher>,
    pub installer: Installer,
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
        let mut pe = None;
        match extension.as_str() {
            MSI => msi = Some(Msi::new(Cursor::new(data.as_ref()))?),
            MSIX | APPX => msix = Some(Msix::new(Cursor::new(data.as_ref()))?),
            MSIX_BUNDLE | APPX_BUNDLE => {
                msix_bundle = Some(MsixBundle::new(Cursor::new(data.as_ref()))?);
            }
            ZIP => zip = Some(Zip::new(Cursor::new(data.as_ref()))?),
            EXE => pe = yara_x::mods::invoke::<PE>(data.as_ref()),
            _ => {}
        }
        let mut inno = None;
        let mut nsis = None;
        let mut installer_type = None;
        let mut pe_arch = None;
        if let Some(ref pe) = pe {
            pe_arch = Some(Architecture::get_from_exe(pe)?);
            installer_type = Some(InstallerType::get(Some(pe), &extension, msi.as_ref())?);
            if let Ok(inno_file) = InnoFile::new(data.as_ref(), pe) {
                inno = Some(inno_file);
                installer_type = Some(InstallerType::Inno);
            } else if let Ok(nsis_file) = Nsis::new(data.as_ref(), pe) {
                nsis = Some(nsis_file);
                installer_type = Some(InstallerType::Nullsoft);
            } else if let Some(msi_resource) = get_msi_resource(pe) {
                installer_type = Some(InstallerType::Burn);
                msi = Some(extract_msi(data.as_ref(), msi_resource)?);
            }
        }
        if installer_type.is_none() {
            installer_type = Some(InstallerType::get(pe.as_deref(), &extension, msi.as_ref())?);
        }
        let upgrade_code = msi.as_mut().map(|msi| mem::take(&mut msi.upgrade_code));
        let display_name = msi
            .as_mut()
            .map(|msi| mem::take(&mut msi.product_name))
            .or_else(|| msix.as_mut().map(|msix| mem::take(&mut msix.display_name)))
            .or_else(|| inno.as_mut().and_then(|inno| inno.uninstall_name.take()))
            .or_else(|| nsis.as_mut().and_then(|nsis| nsis.display_name.take()));
        let display_publisher = msi
            .as_mut()
            .map(|msi| mem::take(&mut msi.manufacturer))
            .or_else(|| {
                msix.as_mut()
                    .map(|msix| mem::take(&mut msix.publisher_display_name))
            })
            .or_else(|| inno.as_mut().and_then(|inno| inno.app_publisher.take()))
            .or_else(|| nsis.as_mut().and_then(|nsis| nsis.display_publisher.take()));
        let display_version = msi
            .as_mut()
            .map(|msi| mem::take(&mut msi.product_version))
            .or_else(|| msix.as_mut().map(|msix| mem::take(&mut msix.version)))
            .or_else(|| inno.as_mut().and_then(|inno| inno.app_version.take()))
            .or_else(|| nsis.as_mut().and_then(|nsis| nsis.display_version.take()))
            .and_then(Versioning::new);
        let product_code = msi
            .as_mut()
            .map(|msi| mem::take(&mut msi.product_code))
            .or_else(|| inno.as_mut().and_then(|inno| inno.product_code.take()));
        let installer = Installer {
            installer_locale: msi
                .as_mut()
                .map(|msi| mem::take(&mut msi.product_language))
                .or_else(|| inno.as_mut().and_then(|inno| inno.installer_locale.take()))
                .or_else(|| {
                    nsis.as_mut()
                        .map(|nsis| mem::take(&mut nsis.install_locale))
                }),
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
                .or_else(|| inno.as_ref().and_then(|inno| inno.architecture))
                .or_else(|| nsis.as_ref().map(|nsis| nsis.architecture))
                .or(pe_arch)
                .or_else(|| zip.as_mut().and_then(|zip| zip.architecture.take()))
                .unwrap_or_default(),
            installer_type,
            nested_installer_type: zip
                .as_mut()
                .and_then(|zip| zip.nested_installer_type.take()),
            nested_installer_files: zip
                .as_mut()
                .and_then(|zip| zip.nested_installer_files.take()),
            scope: msi.as_ref().and_then(|msi| msi.all_users),
            installer_url: DecodedUrl::default(),
            installer_sha_256: Sha256String::default(),
            signature_sha_256: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.signature_sha_256))
                .or_else(|| {
                    msix_bundle
                        .as_mut()
                        .map(|msix_bundle| mem::take(&mut msix_bundle.signature_sha_256))
                }),
            upgrade_behavior: installer_type.and_then(UpgradeBehavior::get),
            file_extensions: msix.as_mut().and_then(|msix| msix.file_extensions.take()),
            package_family_name: msix
                .as_mut()
                .map(|msix| mem::take(&mut msix.package_family_name))
                .or_else(|| msix_bundle.map(|msix_bundle| msix_bundle.package_family_name)),
            product_code: product_code.clone(),
            capabilities: msix.as_mut().and_then(|msix| msix.capabilities.take()),
            restricted_capabilities: msix
                .as_mut()
                .and_then(|msix| msix.restricted_capabilities.take()),
            unsupported_os_architectures: inno
                .as_mut()
                .and_then(|inno| inno.unsupported_architectures.take()),
            apps_and_features_entries: if display_name.is_some()
                || display_publisher.is_some()
                || display_version.is_some()
                || upgrade_code.is_some()
            {
                Some(Vec::from([AppsAndFeaturesEntry {
                    display_name,
                    publisher: display_publisher,
                    display_version,
                    product_code,
                    upgrade_code,
                    ..AppsAndFeaturesEntry::default()
                }]))
            } else {
                None
            },
            elevation_requirement: inno.and_then(|inno| inno.elevation_requirement),
            installation_metadata: msi
                .and_then(|msi| msi.install_location)
                .or_else(|| msix.map(|msix| msix.install_location))
                .or_else(|| nsis.and_then(|nsis| nsis.install_dir))
                .map(|install_location| InstallationMetadata {
                    default_install_location: Some(install_location),
                    ..InstallationMetadata::default()
                }),
            ..Installer::default()
        };
        Ok(Self {
            installer,
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
