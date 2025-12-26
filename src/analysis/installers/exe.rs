use std::io::{Read, Seek};

use color_eyre::Result;
use inno::{Inno, error::InnoError};
use tracing::debug;
use winget_types::installer::{Architecture, Installer, InstallerSwitches, InstallerType};
use yara_x::mods::PE;

use super::{super::Installers, Burn, Nsis};
use crate::{
    analysis::installers::{burn::BurnError, nsis::NsisError},
    traits::FromMachine,
};

const ORIGINAL_FILENAME: &str = "OriginalFilename";
const FILE_DESCRIPTION: &str = "FileDescription";
const BASIC_INSTALLER_KEYWORDS: [&str; 4] = ["installer", "setup", "7zs.sfx", "7zsd.sfx"];

pub enum Exe {
    Burn(Box<Burn>),
    Inno(Box<Inno>),
    Nsis(Nsis),
    Generic(Box<Installer>),
}

impl Exe {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self> {
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

        const INSTALLSHIELD_MAGICS: [&[u8]; 2] = [b"InstallShield", b"ISSetupStream"];
        if let Some(offset) = pe.overlay.offset {
            reader.seek(std::io::SeekFrom::Start(offset))?;

            // Check for optional CV_INFO_PDB20 structure before magic
            // 4 DWORDs: CvSignature, Offset, TimeDateStamp, Age, then null-terminated PdbFileName
            let mut signature = [0u8; 4];
            if reader.read_exact(&mut signature).is_ok() && signature == *b"NB10" {
                reader.seek(std::io::SeekFrom::Current(12))?;
                let mut byte = [0u8; 1];
                while reader.read_exact(&mut byte).is_ok() && byte[0] != 0 {}
            } else {
                reader.seek(std::io::SeekFrom::Start(offset))?;
            }

            let mut magic = [0u8; 13];
            if reader.read_exact(&mut magic).is_ok()
                && INSTALLSHIELD_MAGICS.iter().any(|m| *m == magic)
            {
                debug!("Detected InstallShield exe");
                return Ok(Self::Generic(Box::new(Installer {
                    architecture: Architecture::from_machine(pe.machine()),
                    r#type: Some(InstallerType::Exe),
                    switches: InstallerSwitches::builder()
                        .silent("/S /v/qn".parse().unwrap())
                        .silent_with_progress("/S /v/qn".parse().unwrap())
                        .build(),
                    ..Installer::default()
                })));
            }
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
            Self::Burn(burn) => burn.installers(),
            Self::Inno(inno) => inno.installers(),
            Self::Nsis(nsis) => nsis.installers(),
            Self::Generic(installer) => vec![*installer.clone()],
        }
    }
}
