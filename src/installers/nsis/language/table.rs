use crate::installers::nsis::header::block::BlockType;
use crate::installers::nsis::header::Header;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use itertools::Itertools;
use zerocopy::little_endian::{U16, U32};
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct LanguageTable {
    pub language_id: U16,
    dialog_offset: U32,
    right_to_left: U32,
    pub language_string_offsets: [U32],
}

const EN_US_LANG_CODE: U16 = U16::new(1033);

impl LanguageTable {
    pub fn get_main<'data>(data: &'data [u8], header: &Header) -> Result<&'data Self> {
        BlockType::LangTables
            .get(data, &header.blocks)
            .chunks_exact(header.langtable_size.get() as usize)
            .filter_map(|data| Self::ref_from_bytes(data).ok())
            .find_or_first(|lang_table| lang_table.language_id == EN_US_LANG_CODE)
            .ok_or_eyre("No NSIS language table found")
    }
}
