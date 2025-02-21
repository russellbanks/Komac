use crate::installers::burn::{Burn, BurnError};
use crate::installers::inno::{Inno, InnoError};
use crate::installers::msi::Msi;
use crate::installers::msix_family::bundle::MsixBundle;
use crate::installers::msix_family::Msix;
use crate::installers::nsis::{Nsis, NsisError};
use crate::installers::possible_installers::PossibleInstaller;
use crate::installers::zip::Zip;
use crate::manifests::installer_manifest::Installer;
use crate::types::architecture::Architecture;
use crate::types::copyright::Copyright;
use crate::types::installer_type::InstallerType;
use crate::types::package_name::PackageName;
use crate::types::publisher::Publisher;
use camino::Utf8Path;
use color_eyre::eyre::{bail, Result};
use memmap2::Mmap;
use std::io::Cursor;
use std::mem;
use tracing::debug;
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
    pub installers: Vec<Installer>,
    pub zip: Option<Zip<Cursor<&'data [u8]>>>,
}

impl<'data> FileAnalyser<'data> {
    pub fn new(data: &'data Mmap, file_name: &str) -> Result<Self> {
        let extension = Utf8Path::new(file_name)
            .extension()
            .unwrap_or_default()
            .to_lowercase();
        let mut zip = None;
        let mut copyright = None;
        let mut package_name = None;
        let mut publisher = None;
        let installer = match extension.as_str() {
            MSI => PossibleInstaller::Msi(Msi::new(Cursor::new(data.as_ref()))?),
            MSIX | APPX => PossibleInstaller::Msix(Msix::new(Cursor::new(data.as_ref()))?),
            MSIX_BUNDLE | APPX_BUNDLE => {
                PossibleInstaller::MsixBundle(MsixBundle::new(Cursor::new(data.as_ref()))?)
            }
            ZIP => {
                let mut scoped_zip = Zip::new(Cursor::new(data.as_ref()))?;
                let installer = PossibleInstaller::Zip(mem::take(&mut scoped_zip.installers));
                zip = Some(scoped_zip);
                installer
            }
            EXE => {
                let pe = yara_x::mods::invoke::<PE>(data.as_ref()).unwrap();
                debug!(?pe.version_info);
                copyright = Copyright::get_from_exe(&pe.version_info);
                package_name = PackageName::get_from_exe(&pe.version_info);
                publisher = Publisher::get_from_exe(&pe.version_info);
                match Burn::new(data.as_ref(), &pe) {
                    Ok(burn) => PossibleInstaller::Burn(burn),
                    Err(BurnError::NotBurnFile) => match Nsis::new(data.as_ref(), &pe) {
                        Ok(nsis_file) => PossibleInstaller::Nsis(nsis_file),
                        Err(NsisError::NotNsisFile) => match Inno::new(data.as_ref(), &pe) {
                            Ok(inno_file) => PossibleInstaller::Inno(inno_file),
                            Err(InnoError::NotInnoFile) => PossibleInstaller::Other(Installer {
                                architecture: Architecture::from_machine(pe.machine()),
                                r#type: pe
                                    .version_info_list
                                    .iter()
                                    .filter(|key_value| {
                                        matches!(
                                            key_value.key(),
                                            FILE_DESCRIPTION | ORIGINAL_FILENAME
                                        )
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
                                    .or(Some(InstallerType::Portable)),
                                ..Installer::default()
                            }),
                            Err(inno_error) => return Err(inno_error.into()),
                        },
                        Err(nsis_error) => return Err(nsis_error.into()),
                    },
                    Err(burn_error) => return Err(burn_error.into()),
                }
            }
            _ => bail!(r#"Unsupported file extension: "{extension}""#),
        };
        Ok(Self {
            installers: installer.installers(),
            file_name: String::new(),
            copyright,
            package_name,
            publisher,
            zip,
        })
    }
}
