use crate::installers::nsis::header::block::BlockType;
use crate::installers::nsis::header::Header;
use color_eyre::eyre::{Error, OptionExt};
use color_eyre::Result;
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

const EN_US_LANG_CODE: u16 = 1033;

impl LanguageTable {
    pub fn get_main<'data>(data: &'data [u8], header: &Header) -> Result<&'data Self> {
        let lang_table_block_header = &header.blocks[BlockType::LangTables as usize];

        let mut first_table = None;

        for index in 0..lang_table_block_header.num.get() {
            let offset = lang_table_block_header.offset.get() as usize
                + (header.langtable_size.get() * index) as usize;
            let lang_table = &data[offset..offset + header.langtable_size.get() as usize];
            let table =
                Self::ref_from_bytes(lang_table).map_err(|error| Error::msg(error.to_string()))?;
            if first_table.is_none() {
                first_table = Some(table);
            }
            if table.language_id.get() == EN_US_LANG_CODE {
                return Ok(table);
            }
        }

        first_table.ok_or_eyre("No NSIS language table found")
    }
}
