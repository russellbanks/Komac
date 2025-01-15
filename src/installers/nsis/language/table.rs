use crate::installers::nsis::header::block::{BlockHeaders, BlockType};
use crate::installers::nsis::header::Header;
use itertools::Itertools;
use std::io::{Error, ErrorKind, Result};
use zerocopy::little_endian::{U16, U32};
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct LanguageTable {
    pub id: U16,
    dialog_offset: U32,
    right_to_left: U32,
    pub string_offsets: [U32],
}

const EN_US_LANG_CODE: U16 = U16::new(1033);

impl LanguageTable {
    pub fn get_main<'data>(
        data: &'data [u8],
        header: &Header,
        blocks: &BlockHeaders,
    ) -> Result<&'data Self> {
        BlockType::LangTables
            .get(data, blocks)
            .chunks_exact(header.langtable_size.get() as usize)
            .flat_map(Self::ref_from_bytes)
            .find_or_first(|lang_table| lang_table.id == EN_US_LANG_CODE)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "No NSIS language table found"))
    }
}
