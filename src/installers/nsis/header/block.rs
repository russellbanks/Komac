use std::ops::Index;
use std::slice::Iter;
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
    pub fn get<'data>(self, data: &'data [u8], blocks: &BlockHeaders) -> &'data [u8] {
        let start = blocks[self].offset.get() as usize;
        let end = blocks
            .iter()
            .skip(self as usize + 1)
            .find(|b| b.offset > U32::ZERO)
            .map_or(start, |block| block.offset.get() as usize);
        &data[start..end]
    }
}

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct BlockHeaders([BlockHeader; BlockType::COUNT]);

impl BlockHeaders {
    pub fn iter(&self) -> Iter<BlockHeader> {
        self.0.iter()
    }
}

impl Index<BlockType> for BlockHeaders {
    type Output = BlockHeader;

    fn index(&self, index: BlockType) -> &Self::Output {
        &self.0[index as usize]
    }
}
