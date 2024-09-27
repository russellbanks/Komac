use strum::EnumCount;
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct BlockHeader {
    pub offset: U32,
    pub num: U32,
}

#[expect(dead_code)]
#[derive(Copy, Clone, EnumCount)]
pub enum BlockType {
    Pages,
    Sections,
    Entries,
    Strings,
    LangTables,
    CtlColors,
    BgFont,
    Data,
}

impl BlockType {
    pub fn get<'data>(self, data: &'data [u8], blocks: &[BlockHeader; Self::COUNT]) -> &'data [u8] {
        let start = blocks[self as usize].offset.get() as usize;
        let end = blocks
            .iter()
            .skip(self as usize + 1)
            .find(|b| b.offset.get() > 0)
            .map_or(blocks.len(), |block| block.offset.get() as usize);
        &data[start..end]
    }
}
