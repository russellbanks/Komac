mod entry;
mod first_header;
mod header;
mod language;
mod strings;
mod version;

use crate::installers::nsis::entry::registry::WriteReg;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::Header;
use crate::installers::nsis::language::table::LanguageTable;
use crate::installers::nsis::version::NsisVersion;
use crate::installers::utils::RELATIVE_PROGRAM_FILES_64;
use crate::types::architecture::Architecture;
use crate::types::language_tag::LanguageTag;
use camino::Utf8PathBuf;
use color_eyre::eyre::{Error, OptionExt};
use color_eyre::Result;
use header::block::BlockType;
use msi::Language;
use std::mem;
use std::str::FromStr;
use strings::encoding::nsis_string;
use yara_x::mods::PE;
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, TryFromBytes};

pub struct Nsis {
    pub architecture: Architecture,
    pub install_dir: Option<Utf8PathBuf>,
    pub install_locale: LanguageTag,
    pub display_name: Option<String>,
    pub display_version: Option<String>,
    pub display_publisher: Option<String>,
}

impl Nsis {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        let first_header_offset = pe
            .overlay
            .offset
            .and_then(|offset| usize::try_from(offset).ok())
            .ok_or_eyre("Unable to get NSIS first header offset")?;

        let data_offset = first_header_offset + size_of::<FirstHeader>();
        let first_header = FirstHeader::try_ref_from_bytes(&data[first_header_offset..data_offset])
            .map_err(|error| Error::msg(error.to_string()))?;

        let decompressed_data = Header::decompress(&data[data_offset..], first_header)?;
        let (header, _) = Header::ref_from_prefix(&decompressed_data)
            .map_err(|error| Error::msg(error.to_string()))?;

        let strings_block = BlockType::Strings.get(&decompressed_data, &header.blocks);

        let language_table = LanguageTable::get_main(&decompressed_data, header)?;

        let nsis_version = NsisVersion::from_manifest(data, pe)
            .or_else(|| NsisVersion::from_branding_text(strings_block, language_table))
            .unwrap_or_else(|| NsisVersion::detect(strings_block));

        let install_dir = if header.install_directory_ptr != U32::ZERO {
            nsis_string()
                .strings_block(strings_block)
                .relative_offset(header.install_directory_ptr.get())
                .nsis_version(nsis_version)
                .get()
                .ok()
        } else {
            None
        };

        let architecture = if install_dir
            .as_deref()
            .is_some_and(|dir| dir.contains(RELATIVE_PROGRAM_FILES_64))
        {
            Architecture::X64
        } else {
            Architecture::X86
        };

        let entries = <[Entry]>::try_ref_from_bytes(
            BlockType::Entries.get(&decompressed_data, &header.blocks),
        )
        .map_err(|error| Error::msg(error.to_string()))?;

        let mut write_reg = entries
            .iter()
            .filter_map(|entry| WriteReg::from_entry(entry, strings_block, nsis_version))
            .collect::<Vec<_>>();

        Ok(Self {
            architecture,
            install_dir: install_dir.map(Utf8PathBuf::from),
            install_locale: LanguageTag::from_str(
                Language::from_code(language_table.language_id.get()).tag(),
            )?,
            display_name: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "DisplayName")
                .map(|write_reg| mem::take(&mut write_reg.value)),
            display_version: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "DisplayVersion")
                .map(|write_reg| mem::take(&mut write_reg.value)),
            display_publisher: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "Publisher")
                .map(|write_reg| mem::take(&mut write_reg.value)),
        })
    }
}
