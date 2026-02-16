use std::io::{Read, Seek};

use color_eyre::Result;
use inno::{Inno, error::InnoError};
use winget_types::installer::{Architecture, Installer, InstallerType};
use yara_x::mods::PE;

use super::{super::Installers, AdvancedInstaller, Burn, Nsis};
use crate::{
    analysis::installers::{advanced::AdvancedInstallerError, burn::BurnError, nsis::NsisError},
    traits::FromMachine,
};

const ORIGINAL_FILENAME: &str = "OriginalFilename";
const FILE_DESCRIPTION: &str = "FileDescription";
const BASIC_INSTALLER_KEYWORDS: [&str; 4] = ["installer", "setup", "7zs.sfx", "7zsd.sfx"];

pub enum Exe {
    AdvancedInstaller(Box<AdvancedInstaller>),
    Burn(Box<Burn>),
    Inno(Box<Inno>),
    Nsis(Nsis),
    Generic(Box<Installer>),
}

impl Exe {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self> {
        match AdvancedInstaller::new(&mut reader, pe) {
            Ok(advanced) => return Ok(Self::AdvancedInstaller(Box::new(advanced))),
            Err(AdvancedInstallerError::NotAdvancedInstallerFile) => {}
            Err(error) => return Err(error.into()),
        }

        match Burn::new(&mut reader, pe) {
            Ok(burn) => return Ok(Self::Burn(Box::new(burn))),
            Err(BurnError::NotBurnFile) => {}
            Err(error) => return Err(error.into()),
        }

        match Inno::new(&mut reader) {
            Ok(inno) => return Ok(Self::Inno(Box::new(inno))),
            Err(InnoError::NotInnoFile) => {}
            Err(error) => return Err(error.into()),
        }

        match Nsis::new(&mut reader, pe) {
            Ok(nsis) => return Ok(Self::Nsis(nsis)),
            Err(NsisError::NotNsisFile) => {}
            Err(error) => return Err(error.into()),
        }

        Ok(Self::Generic(Box::new(Installer {
            architecture: Architecture::from_machine(pe.machine()),
            r#type: if pe
                .version_info_list
                .iter()
                .filter(|key_value| matches!(key_value.key(), FILE_DESCRIPTION | ORIGINAL_FILENAME))
                .filter_map(|key_value| key_value.value.as_deref().map(str::to_ascii_lowercase))
                .any(|value| {
                    BASIC_INSTALLER_KEYWORDS
                        .iter()
                        .any(|keyword| value.contains(keyword))
                }) {
                Some(InstallerType::Exe)
            } else {
                Some(InstallerType::Portable)
            },
            ..Installer::default()
        })))
    }
}

impl Installers for Exe {
    fn installers(&self) -> Vec<Installer> {
        match self {
            Self::AdvancedInstaller(advanced) => advanced.installers(),
            Self::Burn(burn) => burn.installers(),
            Self::Inno(inno) => inno.installers(),
            Self::Nsis(nsis) => nsis.installers(),
            Self::Generic(installer) => vec![*installer.clone()],
        }
    }
}
