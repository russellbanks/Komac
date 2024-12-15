use std::io::Cursor;

use crate::installers::burn::Burn;
use crate::installers::inno::{Inno, InnoError};
use crate::installers::msi::Msi;
use crate::installers::msix_family::bundle::MsixBundle;
use crate::installers::msix_family::Msix;
use crate::installers::nsis::{Nsis, NsisError};
use crate::installers::traits::InstallSpec;
use crate::installers::zip::Zip;
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, InstallationMetadata, Installer, UpgradeBehavior,
};
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::installer_type::InstallerType;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use camino::Utf8Path;
use color_eyre::eyre::{bail, Error, Result};
use memmap2::Mmap;
use versions::Versioning;
use yara_x::mods::PE;

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";

const ORIGINAL_FILENAME: &str = "OriginalFilename";
const FILE_DESCRIPTION: &str = "FileDescription";
const BASIC_INSTALLER_KEYWORDS: [&str; 4] = ["installer", "setup", "7zs.sfx", "7zsd.sfx"];

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
        let mut zip = None;
        let mut pe = None;
        let mut installer_type = None;
        let mut installer: Option<Box<dyn InstallSpec>> = None;
        match extension.as_str() {
            MSI => installer = Some(Box::new(Msi::new(Cursor::new(data.as_ref()))?)),
            MSIX | APPX => installer = Some(Box::new(Msix::new(Cursor::new(data.as_ref()))?)),
            MSIX_BUNDLE | APPX_BUNDLE => {
                installer = Some(Box::new(MsixBundle::new(Cursor::new(data.as_ref()))?));
            }
            ZIP => {
                zip = Some(Zip::new(Cursor::new(data.as_ref()))?);
                installer_type = Some(InstallerType::Zip);
            }
            EXE => {
                pe = yara_x::mods::invoke::<PE>(data.as_ref());
                if let Some(ref pe) = pe {
                    if let Ok(burn) = Burn::new(data.as_ref(), pe) {
                        installer = Some(Box::new(burn));
                    } else {
                        match Nsis::new(data.as_ref(), pe) {
                            Ok(nsis_file) => installer = Some(Box::new(nsis_file)),
                            Err(error) => match error {
                                NsisError::NotNsisFile => {}
                                _ => return Err(Error::new(error)),
                            },
                        }
                    }

                    if installer.is_none() {
                        match Inno::new(data.as_ref(), pe) {
                            Ok(inno_file) => installer = Some(Box::new(inno_file)),
                            Err(error) => match error {
                                InnoError::NotInnoFile => {}
                                _ => return Err(Error::new(error)),
                            },
                        }
                    }

                    if installer.is_none() {
                        installer_type = pe
                            .version_info_list
                            .iter()
                            .filter(|key_value| {
                                matches!(key_value.key(), FILE_DESCRIPTION | ORIGINAL_FILENAME)
                            })
                            .filter_map(|key_value| {
                                key_value.value.as_deref().map(str::to_ascii_lowercase)
                            })
                            .any(|value| {
                                BASIC_INSTALLER_KEYWORDS
                                    .iter()
                                    .any(|keyword| value.contains(keyword))
                            })
                            .then_some(InstallerType::Exe)
                            .or(Some(InstallerType::Portable));
                    }
                }
            }
            _ => bail!(r#"Unsupported file extension: "{extension}""#),
        }
        let upgrade_code = installer.as_deref_mut().and_then(InstallSpec::upgrade_code);
        let display_name = installer.as_deref_mut().and_then(InstallSpec::display_name);
        let display_publisher = installer
            .as_deref_mut()
            .and_then(InstallSpec::display_publisher);
        let display_version = installer
            .as_deref_mut()
            .and_then(InstallSpec::display_version)
            .and_then(Versioning::new);
        let product_code = installer.as_deref_mut().and_then(InstallSpec::product_code);
        let installer = Installer {
            installer_locale: installer.as_deref_mut().and_then(InstallSpec::locale),
            platform: installer.as_deref_mut().and_then(InstallSpec::platform),
            minimum_os_version: installer.as_deref().and_then(InstallSpec::min_version),
            architecture: installer
                .as_deref_mut()
                .and_then(InstallSpec::architecture)
                .or_else(|| {
                    pe.as_deref()
                        .and_then(|pe| Architecture::from_machine(pe.machine()).ok())
                })
                .unwrap_or_default(),
            installer_type: installer_type
                .or_else(|| installer.as_deref().map(InstallSpec::r#type)),
            nested_installer_type: zip
                .as_mut()
                .and_then(|zip| zip.nested_installer_type.take()),
            nested_installer_files: zip
                .as_mut()
                .and_then(|zip| zip.nested_installer_files.take()),
            scope: installer.as_deref().and_then(InstallSpec::scope),
            signature_sha_256: installer
                .as_deref_mut()
                .and_then(InstallSpec::signature_sha_256),
            upgrade_behavior: installer_type.and_then(UpgradeBehavior::get),
            file_extensions: installer
                .as_deref_mut()
                .and_then(InstallSpec::file_extensions),
            package_family_name: installer
                .as_deref_mut()
                .and_then(InstallSpec::package_family_name),
            product_code: product_code.clone(),
            capabilities: installer.as_deref_mut().and_then(InstallSpec::capabilities),
            restricted_capabilities: installer
                .as_deref_mut()
                .and_then(InstallSpec::restricted_capabilities),
            unsupported_os_architectures: installer
                .as_deref_mut()
                .and_then(InstallSpec::unsupported_os_architectures),
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
            elevation_requirement: installer
                .as_deref_mut()
                .and_then(InstallSpec::elevation_requirement),
            installation_metadata: installer
                .as_deref_mut()
                .and_then(InstallSpec::install_location)
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
