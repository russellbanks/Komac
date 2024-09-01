use std::collections::HashMap;

use color_eyre::eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use yara_x::mods::PE;

use crate::file_analyser::{APPX, APPX_BUNDLE, EXE, MSI, MSIX, MSIX_BUNDLE, ZIP};
use crate::installers::msi::Msi;
use crate::manifests::installer_manifest::NestedInstallerType;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum InstallerType {
    Msix,
    Msi,
    Appx,
    Exe,
    Zip,
    Inno,
    Nullsoft,
    Wix,
    Burn,
    Pwa,
    Portable,
}

impl InstallerType {
    pub fn get(pe: Option<&PE>, extension: &str, msi: Option<&Msi>) -> Result<Self> {
        match extension {
            MSI => {
                if let Some(msi) = msi {
                    return Ok(if msi.is_wix { Self::Wix } else { Self::Msi });
                }
            }
            MSIX | MSIX_BUNDLE => return Ok(Self::Msix),
            APPX | APPX_BUNDLE => return Ok(Self::Appx),
            ZIP => return Ok(Self::Zip),
            EXE => {
                return match () {
                    () if pe.is_some_and(Self::is_burn) => Ok(Self::Burn),
                    () if pe
                        .map(|pe| &pe.version_info)
                        .is_some_and(Self::is_basic_installer) =>
                    {
                        Ok(Self::Exe)
                    }
                    () => Ok(Self::Portable),
                };
            }
            _ => {}
        }
        bail!("Unsupported file extension {extension}")
    }

    fn is_burn(pe: &PE) -> bool {
        const WIXBURN_HEADER: &[u8] = b".wixburn";

        pe.sections
            .iter()
            .any(|section| section.name() == WIXBURN_HEADER)
    }

    fn is_basic_installer(vs_version_info: &HashMap<String, String>) -> bool {
        const ORIGINAL_FILENAME: &str = "OriginalFilename";
        const FILE_DESCRIPTION: &str = "FileDescription";
        const BASIC_INSTALLER_KEYWORDS: [&str; 3] = ["installer", "setup", "7zs.sfx"];

        vs_version_info
            .iter()
            .filter(|&(key, _value)| key == FILE_DESCRIPTION || key == ORIGINAL_FILENAME)
            .map(|(_key, value)| value.to_ascii_lowercase())
            .any(|value| {
                BASIC_INSTALLER_KEYWORDS
                    .iter()
                    .any(|keyword| value.contains(keyword))
            })
    }

    pub const fn to_nested(self) -> Option<NestedInstallerType> {
        match self {
            Self::Msix => Some(NestedInstallerType::Msix),
            Self::Msi => Some(NestedInstallerType::Msi),
            Self::Appx => Some(NestedInstallerType::Appx),
            Self::Exe => Some(NestedInstallerType::Exe),
            Self::Inno => Some(NestedInstallerType::Inno),
            Self::Nullsoft => Some(NestedInstallerType::Nullsoft),
            Self::Wix => Some(NestedInstallerType::Wix),
            Self::Burn => Some(NestedInstallerType::Burn),
            Self::Portable => Some(NestedInstallerType::Portable),
            _ => None,
        }
    }
}
