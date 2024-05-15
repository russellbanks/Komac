use std::collections::HashMap;

use color_eyre::eyre::{bail, Result};
use quick_xml::de::from_reader;
use serde::{Deserialize, Serialize};
use yara_x::mods::pe::ResourceType;
use yara_x::mods::PE;

use crate::file_analyser::{APPX, APPX_BUNDLE, EXE, MSI, MSIX, MSIX_BUNDLE, ZIP};
use crate::manifests::installer_manifest::NestedInstallerType;
use crate::msi::Msi;

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
    pub fn get(
        data: &[u8],
        pe: Option<&Box<PE>>,
        extension: &str,
        msi: Option<&Msi>,
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
                let vs_version_info = pe.map(|pe| &pe.version_info);
                return match () {
                    () if pe.is_some_and(|pe| Self::is_nullsoft(data, pe)) => Ok(Self::Nullsoft),
                    () if vs_version_info.is_some_and(|map| Self::is_inno(map)) => Ok(Self::Inno),
                    () if pe.is_some_and(|pe| Self::is_burn(pe)) => Ok(Self::Burn),
                    () if vs_version_info.is_some_and(|map| Self::is_basic_installer(map)) => {
                        Ok(Self::Exe)
                    }
                    () => Ok(Self::Portable),
                };
            }
            _ => {}
        }
        bail!("Unsupported file extension {extension}")
    }

    /// Checks if the file is Nullsoft from the executable's manifest
    fn is_nullsoft(data: &[u8], pe: &Box<PE>) -> bool {
        #[derive(Default, Deserialize)]
        #[serde(default, rename_all = "camelCase")]
        struct Assembly {
            assembly_identity: AssemblyIdentity,
        }

        #[derive(Default, Deserialize)]
        #[serde(default)]
        struct AssemblyIdentity {
            #[serde(rename = "@name")]
            name: String,
        }

        const NULLSOFT_MANIFEST_NAME: &str = "Nullsoft.NSIS.exehead";

        pe.resources
            .iter()
            .find(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_MANIFEST)
            .and_then(|manifest| {
                let offset = manifest.offset() as usize;
                data.get(offset..offset + manifest.length() as usize)
            })
            .and_then(|manifest_data| from_reader::<_, Assembly>(manifest_data).ok())
            .is_some_and(|assembly| assembly.assembly_identity.name == NULLSOFT_MANIFEST_NAME)
    }

    /// Checks the String File Info of the exe for whether its comment states that it was built with Inno Setup
    fn is_inno(vs_version_info: &HashMap<String, String>) -> bool {
        const COMMENTS: &str = "Comments";
        const INNO_COMMENT: &str = "This installation was built with Inno Setup.";

        vs_version_info.get(COMMENTS).map(String::as_ref) == Some(INNO_COMMENT)
    }

    fn is_burn(pe: &Box<PE>) -> bool {
        const WIXBURN_HEADER: &[u8] = b".wixburn";

        pe.sections
            .iter()
            .find(|section| section.name() == WIXBURN_HEADER)
            .is_some()
    }

    fn is_basic_installer(vs_version_info: &HashMap<String, String>) -> bool {
        const ORIGINAL_FILENAME: &str = "OriginalFilename";
        const FILE_DESCRIPTION: &str = "FileDescription";
        const BASIC_INSTALLER_KEYWORDS: [&str; 3] = ["installer", "setup", "7zs.sfx"];

        vs_version_info
            .iter()
            .filter_map(|(key, value)| {
                (key == FILE_DESCRIPTION || key == ORIGINAL_FILENAME)
                    .then(|| value.to_ascii_lowercase())
            })
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
