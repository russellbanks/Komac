use color_eyre::eyre::{bail, OptionExt, Result};
use object::pe::RT_MANIFEST;
use object::read::pe::{ImageNtHeaders, PeFile, ResourceDirectoryEntryData};
use object::{LittleEndian, Object, ReadRef};
use quick_xml::de::from_reader;
use serde::{Deserialize, Serialize};

use crate::exe::vs_version_info::VSVersionInfo;
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
    pub fn get<'data, Pe, R>(
        pe: Option<&PeFile<'data, Pe, R>>,
        extension: &str,
        msi: Option<&Msi>,
    ) -> Result<Self>
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'data>,
    {
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
                let vs_version_info = pe.and_then(|pe| VSVersionInfo::parse(pe).ok());
                return match () {
                    () if Self::is_inno(vs_version_info.as_ref()) => Ok(Self::Inno),
                    () if pe
                        .and_then(|pe| Self::is_nullsoft(pe).ok())
                        .unwrap_or(false) =>
                    {
                        Ok(Self::Nullsoft)
                    }
                    () if pe.is_some_and(|pe| Self::is_burn(pe)) => Ok(Self::Burn),
                    () if Self::is_basic_installer(vs_version_info.as_ref()) => Ok(Self::Exe),
                    () => Ok(Self::Portable),
                };
            }
            _ => {}
        }
        bail!("Unsupported file extension {extension}")
    }

    /// Checks if the file is Nullsoft from the executable's manifest
    fn is_nullsoft<'data, Pe, R>(pe: &PeFile<'data, Pe, R>) -> Result<bool>
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'data>,
    {
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

        let resource_directory = pe
            .data_directories()
            .resource_directory(pe.data(), &pe.section_table())?
            .ok_or_eyre("No resource directory was found")?;
        let rt_manifest = resource_directory
            .root()?
            .entries
            .iter()
            .find(|entry| entry.name_or_id().id() == Some(RT_MANIFEST))
            .ok_or_eyre("No RT_MANIFEST was found")?
            .data(resource_directory)?
            .table()
            .and_then(|table| table.entries.first())
            .and_then(|entry| entry.data(resource_directory).ok())
            .and_then(ResourceDirectoryEntryData::table)
            .and_then(|table| table.entries.first())
            .and_then(|entry| entry.data(resource_directory).ok())
            .and_then(ResourceDirectoryEntryData::data)
            .unwrap();
        let section = pe
            .section_table()
            .section_containing(rt_manifest.offset_to_data.get(LittleEndian))
            .unwrap();
        let offset = {
            let mut rva = rt_manifest.offset_to_data.get(LittleEndian);
            rva -= section.virtual_address.get(LittleEndian);
            rva += section.pointer_to_raw_data.get(LittleEndian);
            rva as usize
        };
        let manifest = pe
            .data()
            .read_bytes_at(offset as u64, u64::from(rt_manifest.size.get(LittleEndian)))
            .unwrap();
        let assembly: Assembly = from_reader(manifest)?;
        Ok(assembly.assembly_identity.name == NULLSOFT_MANIFEST_NAME)
    }

    /// Checks the String File Info of the exe for whether its comment states that it was built with Inno Setup
    fn is_inno(vs_version_info: Option<&VSVersionInfo>) -> bool {
        const COMMENTS: &str = "Comments";
        const INNO_COMMENT: &str = "This installation was built with Inno Setup.";

        vs_version_info
            .and_then(|info| info.string_file_info.as_ref())
            .is_some_and(|string_info| {
                string_info.children.first().is_some_and(|vs_string_table| {
                    vs_string_table
                        .children
                        .iter()
                        .find(|entry| String::from_utf16_lossy(entry.header.key) == COMMENTS)
                        .map(|entry| String::from_utf16_lossy(entry.value))
                        .as_deref()
                        == Some(INNO_COMMENT)
                })
            })
    }

    fn is_burn<'data, Pe, R>(pe: &PeFile<'data, Pe, R>) -> bool
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'data>,
    {
        const WIXBURN_HEADER: &str = ".wixburn";

        pe.section_by_name(WIXBURN_HEADER).is_some()
    }

    fn is_basic_installer(vs_version_info: Option<&VSVersionInfo>) -> bool {
        const ORIGINAL_FILENAME: &str = "OriginalFilename";
        const FILE_DESCRIPTION: &str = "FileDescription";
        const BASIC_INSTALLER_KEYWORDS: [&str; 3] = ["installer", "setup", "7zs.sfx"];

        vs_version_info
            .and_then(|info| info.string_file_info.as_ref())
            .is_some_and(|string_info| {
                string_info.children.first().is_some_and(|vs_string_table| {
                    vs_string_table
                        .children
                        .iter()
                        .filter(|entry| {
                            let key = String::from_utf16_lossy(entry.header.key);
                            key == FILE_DESCRIPTION || key == ORIGINAL_FILENAME
                        })
                        .map(|entry| {
                            let mut value = String::from_utf16_lossy(entry.value);
                            value.make_ascii_lowercase();
                            value
                        })
                        .any(|value| {
                            BASIC_INSTALLER_KEYWORDS
                                .iter()
                                .any(|keyword| value.contains(keyword))
                        })
                })
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
