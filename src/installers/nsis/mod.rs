mod first_header;
mod header;
mod language;
mod strings;
mod version;

use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::Header;
use crate::installers::nsis::language::table::LanguageTable;
use crate::installers::nsis::version::NsisVersion;
use crate::installers::utils::RELATIVE_PROGRAM_FILES_64;
use crate::types::architecture::Architecture;
use crate::types::language_tag::LanguageTag;
use byteorder::{ByteOrder, LE};
use camino::Utf8PathBuf;
use color_eyre::eyre::{ensure, Error, OptionExt};
use color_eyre::Result;
use header::block::BlockType;
use msi::Language;
use std::str::FromStr;
use strings::encoding::nsis_string;
use yara_x::mods::PE;
use zerocopy::little_endian::U32;
use zerocopy::FromBytes;

pub struct Nsis {
    pub architecture: Architecture,
    pub install_dir: Option<Utf8PathBuf>,
    pub install_locale: LanguageTag,
}

impl Nsis {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        // The first header is positioned after the PE data
        let first_header_offset = pe
            .sections
            .iter()
            .max_by_key(|section| section.raw_data_offset())
            .map(|section| section.raw_data_offset() as usize + section.raw_data_size() as usize)
            .map(|offset| offset.next_multiple_of(FirstHeader::ALIGNMENT as usize))
            .ok_or_eyre("Unable to get NSIS first header offset")?;

        let data_offset = first_header_offset + size_of::<FirstHeader>();
        let first_header = FirstHeader::ref_from_bytes(&data[first_header_offset..data_offset])
            .map_err(|error| Error::msg(error.to_string()))?;

        ensure!(
            first_header.signature.is_valid(),
            "Invalid NSIS header signature"
        );

        let decompressed_data = Header::decompress(&data[data_offset..], first_header)?;
        let (header, _) = Header::ref_from_prefix(&decompressed_data)
            .map_err(|error| Error::msg(error.to_string()))?;

        let strings_block = BlockType::Strings.get(&decompressed_data, &header.blocks);
        let unicode = LE::read_u16(strings_block) == 0;

        let nsis_version = NsisVersion::from_manifest(data, pe)
            .unwrap_or_else(|| NsisVersion::detect(strings_block, unicode));

        let language_table = LanguageTable::get_main(&decompressed_data, header)?;

        let install_dir = if header.install_directory_ptr != U32::ZERO {
            nsis_string()
                .strings_block(strings_block)
                .relative_offset(header.install_directory_ptr.get())
                .nsis_version(nsis_version)
                .unicode(unicode)
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

        Ok(Self {
            architecture,
            install_dir: install_dir.map(Utf8PathBuf::from),
            install_locale: LanguageTag::from_str(
                Language::from_code(language_table.language_id.get()).tag(),
            )?,
        })
    }
}
