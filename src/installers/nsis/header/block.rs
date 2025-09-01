use std::{borrow::Cow, io::Cursor, ops::Index, slice};

use strum::EnumCount;
use winget_types::installer::Architecture;
use zerocopy::{
    FromBytes, Immutable, KnownLayout,
    little_endian::{U32, U64},
};

use crate::installers::nsis::{NsisError, section::Section};

#[derive(Clone, Debug, Default, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct BlockHeader {
    pub offset: U64,
    pub num: U32,
}

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
        let start = usize::try_from(blocks[self].offset.get()).unwrap();
        let end = blocks
            .iter()
            .skip(self as usize + 1)
            .find(|b| b.offset > U64::ZERO)
            .map_or(start, |block| usize::try_from(block.offset.get()).unwrap());
        &data[start..end]
    }
}

#[derive(Clone, Debug, Default, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct BlockHeaders([BlockHeader; BlockType::COUNT]);

impl BlockHeaders {
    /// If the NSIS installer is 64-bit, the offset value in the `BlockHeader` is a u64 rather than
    /// a u32. This aims to still use zerocopy as much as possible, although the data will need to
    /// be owned if the offsets are u32's.
    pub fn read_dynamic_from_prefix(
        data: &[u8],
        architecture: Architecture,
    ) -> Result<(Cow<Self>, &[u8]), NsisError> {
        if architecture.is_64_bit() {
            Self::ref_from_prefix(data)
                .map(|(headers, rest)| (Cow::Borrowed(headers), rest))
                .map_err(|error| NsisError::ZeroCopy(error.to_string()))
        } else {
            let mut reader = Cursor::new(data);
            let mut block_headers = Self::default();
            for header in &mut block_headers {
                *header = BlockHeader {
                    offset: U64::from(U32::read_from_io(&mut reader)?),
                    num: U32::read_from_io(&mut reader)?,
                }
            }
            Ok((
                Cow::Owned(block_headers),
                &data[usize::try_from(reader.position()).unwrap_or_default()..],
            ))
        }
    }

    pub fn sections<'data>(&self, data: &'data [u8]) -> impl Iterator<Item = &'data Section> {
        let sections = BlockType::Sections.get(data, self);
        let section_size = sections.len() / self[BlockType::Sections].num.get() as usize;
        sections
            .chunks_exact(section_size)
            .flat_map(Section::ref_from_bytes)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &BlockHeader> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BlockHeader> {
        self.into_iter()
    }
}

impl Index<BlockType> for BlockHeaders {
    type Output = BlockHeader;

    fn index(&self, index: BlockType) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl<'a> IntoIterator for &'a BlockHeaders {
    type Item = &'a BlockHeader;

    type IntoIter = slice::Iter<'a, BlockHeader>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut BlockHeaders {
    type Item = &'a mut BlockHeader;

    type IntoIter = slice::IterMut<'a, BlockHeader>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
