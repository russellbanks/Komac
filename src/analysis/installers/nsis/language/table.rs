use std::io::{Error, ErrorKind, Result};

use itertools::Itertools;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LE, U16, U32};

use crate::analysis::installers::nsis::header::{
    Header,
    block::{BlockHeaders, BlockType},
};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct LanguageTable {
    id: U16<LE>,
    dialog_offset: U32<LE>,
    right_to_left: U32<LE>,
    pub string_offsets: [I32<LE>],
}

const EN_US_LANG_CODE: U16<LE> = U16::new(1033);

impl LanguageTable {
    pub fn primary_language<'data>(
        data: &'data [u8],
        header: &Header,
        blocks: &BlockHeaders,
    ) -> Result<&'data Self> {
        BlockType::LangTables
            .get(data, blocks)
            .chunks_exact(header.language_table_size().unsigned_abs() as usize)
            .flat_map(Self::ref_from_bytes)
            .find_or_first(|lang_table| lang_table.id == EN_US_LANG_CODE)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "No NSIS language table found"))
    }

    #[inline]
    pub const fn id(&self) -> u16 {
        self.id.get()
    }

    #[expect(unused)]
    #[inline]
    pub const fn dialog_offset(&self) -> u32 {
        self.dialog_offset.get()
    }

    #[expect(unused)]
    #[inline]
    pub fn right_to_left(&self) -> bool {
        self.right_to_left != U32::ZERO
    }
}
