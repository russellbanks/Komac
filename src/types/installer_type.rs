use crate::file_analyser::{APPX, APPX_BUNDLE, EXE, MSI, MSIX, MSIX_BUNDLE, ZIP};
use crate::manifests::installer_manifest::NestedInstallerType;
use crate::msi::Msi;
use async_tempfile::TempFile;
use color_eyre::eyre::{bail, Result};
use exe::ResolvedDirectoryID::{Name, ID};
use exe::{ResourceDirectory, VSVersionInfo, VecPE, WCharString};
use serde::{Deserialize, Serialize};
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

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
    pub async fn get(
        file: &mut TempFile,
        extension: &str,
        msi: Option<&Msi>,
        pe: Option<&VecPE>,
    ) -> Result<Self> {
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
                    () if pe.is_some_and(Self::is_inno) => Ok(Self::Inno),
                    () if Self::is_nullsoft(file).await? => Ok(Self::Nullsoft),
                    () if pe.is_some_and(Self::is_burn) => Ok(Self::Burn),
                    () => Ok(Self::Exe),
                }
            }
            _ => {}
        }
        bail!("Unsupported file extension {extension}")
    }

    /// Checks if the file is Nullsoft from its magic bytes
    async fn is_nullsoft(file: &mut TempFile) -> Result<bool> {
        const NULLSOFT_BYTES_LEN: usize = 224;

        /// The first 224 bytes of an exe made with NSIS are always the same
        const NULLSOFT_BYTES: [u8; NULLSOFT_BYTES_LEN] = [
            77, 90, 144, 0, 3, 0, 0, 0, 4, 0, 0, 0, 255, 255, 0, 0, 184, 0, 0, 0, 0, 0, 0, 0, 64,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 216, 0, 0, 0, 14, 31, 186, 14, 0, 180, 9, 205, 33, 184, 1, 76, 205,
            33, 84, 104, 105, 115, 32, 112, 114, 111, 103, 114, 97, 109, 32, 99, 97, 110, 110, 111,
            116, 32, 98, 101, 32, 114, 117, 110, 32, 105, 110, 32, 68, 79, 83, 32, 109, 111, 100,
            101, 46, 13, 13, 10, 36, 0, 0, 0, 0, 0, 0, 0, 173, 49, 8, 129, 233, 80, 102, 210, 233,
            80, 102, 210, 233, 80, 102, 210, 42, 95, 57, 210, 235, 80, 102, 210, 233, 80, 103, 210,
            76, 80, 102, 210, 42, 95, 59, 210, 230, 80, 102, 210, 189, 115, 86, 210, 227, 80, 102,
            210, 46, 86, 96, 210, 232, 80, 102, 210, 82, 105, 99, 104, 233, 80, 102, 210, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 69, 0, 0, 76, 1, 5,
            0,
        ];

        let mut buffer = [0; NULLSOFT_BYTES_LEN];
        file.seek(SeekFrom::Start(0)).await?;
        file.read_exact(&mut buffer).await?;
        Ok(buffer == NULLSOFT_BYTES)
    }

    /// Checks the String File Info of the exe for whether its comment states that it was built with Inno Setup
    fn is_inno(pe: &VecPE) -> bool {
        const COMMENTS: &str = "Comments";
        const INNO_COMMENT: &str = "This installation was built with Inno Setup.";

        VSVersionInfo::parse(pe)
            .ok()
            .and_then(|info| info.string_file_info)
            .is_some_and(|mut string_info| {
                string_info
                    .children
                    .swap_remove(0)
                    .children
                    .into_iter()
                    .find(|entry| {
                        entry
                            .header
                            .key
                            .as_u16_str()
                            .map(|utf16_str| utf16_str.to_string())
                            .ok()
                            .as_deref()
                            == Some(COMMENTS)
                    })
                    .and_then(|entry| {
                        entry
                            .value
                            .as_u16_str()
                            .map(|utf_16_str| utf_16_str.to_string())
                            .ok()
                    })
                    .as_deref()
                    == Some(INNO_COMMENT)
            })
    }

    fn is_burn(pe: &VecPE) -> bool {
        ResourceDirectory::parse(pe)
            .map(|resource_directory| {
                resource_directory
                    .resources
                    .into_iter()
                    .any(|entry| match entry.rsrc_id {
                        Name(mut value) => {
                            value.make_ascii_lowercase();
                            value == MSI
                        }
                        ID(_) => false,
                    })
            })
            .unwrap_or(false)
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
