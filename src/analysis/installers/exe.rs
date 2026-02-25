use std::io::{Read, Seek};

use color_eyre::Result;
use inno::{Inno, error::InnoError};
use winget_types::installer::{Installer, InstallerType};

use super::{super::Installers, Burn, Nsis};
use crate::{
    analysis::installers::{
        burn::BurnError,
        nsis::NsisError,
        pe::{PE, VSVersionInfo},
    },
    traits::IntoWingetArchitecture,
};

const ORIGINAL_FILENAME: &str = "OriginalFilename";
const FILE_DESCRIPTION: &str = "FileDescription";
const BASIC_INSTALLER_KEYWORDS: [&str; 4] = ["installer", "setup", "7zs.sfx", "7zsd.sfx"];

pub struct Exe {
    r#type: ExeType,
    pub legal_copyright: Option<String>,
    pub product_name: Option<String>,
    pub company_name: Option<String>,
}

pub enum ExeType {
    Burn(Box<Burn>),
    Inno(Box<Inno>),
    Nsis(Nsis),
    Generic(Box<Installer>),
}

impl Exe {
    pub fn new<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let pe = PE::read_from(&mut reader)?;

        let vs_version_info_bytes = pe.vs_version_info(&mut reader)?;
        let vs_version_info = VSVersionInfo::read_from(&vs_version_info_bytes)?;
        let mut string_table = vs_version_info.string_table();
        let legal_copyright = string_table.remove("LegalCopyright").map(str::to_owned);
        let product_name = string_table.remove("ProductName").map(str::to_owned);
        let company_name = string_table.remove("CompanyName").map(str::to_owned);

        match Burn::new(&mut reader, &pe) {
            Ok(burn) => {
                return Ok(Self {
                    r#type: ExeType::Burn(Box::new(burn)),
                    legal_copyright,
                    product_name,
                    company_name,
                });
            }
            Err(BurnError::NotBurnFile) => {}
            Err(error) => return Err(error.into()),
        }

        match Inno::new(&mut reader) {
            Ok(inno) => {
                return Ok(Self {
                    r#type: ExeType::Inno(Box::new(inno)),
                    legal_copyright,
                    product_name,
                    company_name,
                });
            }
            Err(InnoError::NotInnoFile) => {}
            Err(error) => return Err(error.into()),
        }

        match Nsis::new(&mut reader, &pe) {
            Ok(nsis) => {
                return Ok(Self {
                    r#type: ExeType::Nsis(nsis),
                    legal_copyright,
                    product_name,
                    company_name,
                });
            }
            Err(NsisError::NotNsisFile) => {}
            Err(error) => return Err(error.into()),
        }

        Ok(Self {
            r#type: ExeType::Generic(Box::new(Installer {
                architecture: pe.winget_architecture(),
                r#type: if vs_version_info
                    .string_entries()
                    .filter(|&(key, _value)| matches!(key, FILE_DESCRIPTION | ORIGINAL_FILENAME))
                    .map(|(_key, value)| value.to_ascii_lowercase())
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
            })),
            legal_copyright,
            product_name,
            company_name,
        })
    }
}

impl Installers for Exe {
    fn installers(&self) -> Vec<Installer> {
        match &self.r#type {
            ExeType::Burn(burn) => burn.installers(),
            ExeType::Inno(inno) => inno.installers(),
            ExeType::Nsis(nsis) => nsis.installers(),
            ExeType::Generic(installer) => vec![*installer.clone()],
        }
    }
}
