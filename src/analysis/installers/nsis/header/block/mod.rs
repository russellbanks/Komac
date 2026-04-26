mod r#type;

use std::{borrow::Cow, fmt, io::Cursor, ops::Index, slice};

use strum::{EnumCount, IntoEnumIterator};
pub use r#type::BlockType;
use zerocopy::{
    FromBytes, Immutable, KnownLayout, TryFromBytes,
    little_endian::{U32, U64},
};

use super::super::{Entry, NsisError, language::table::LanguageTable, section::Section};

#[derive(Clone, Copy, Default, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct BlockHeader {
    pub offset: U64,
    num: U32,
}

impl BlockHeader {
    #[inline]
    pub const fn offset(&self) -> u64 {
        self.offset.get()
    }

    #[inline]
    pub const fn num(&self) -> u32 {
        self.num.get()
    }
}

impl fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockHeader")
            .field("offset", &self.offset())
            .field("num", &self.num())
            .finish()
    }
}

#[derive(Clone, Default, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct BlockHeaders([BlockHeader; BlockType::COUNT]);

impl BlockHeaders {
    /// If the NSIS installer is 64-bit, the offset value in the `BlockHeader` is a u64 rather than
    /// a u32. This aims to still use zerocopy as much as possible, although the data will need to
    /// be owned if the offsets are u32's.
    pub fn read_dynamic_from_prefix(
        data: &[u8],
        is_64_bit: bool,
    ) -> Result<(Cow<'_, Self>, &[u8]), NsisError> {
        if is_64_bit {
            Self::ref_from_prefix(data)
                .map(|(headers, rest)| (Cow::Borrowed(headers), rest))
                .map_err(|error| NsisError::ZeroCopy(error.to_string()))
        } else {
            let mut reader = Cursor::new(data);
            let mut block_headers = Self::default();
            for header in &mut block_headers {
                *header = BlockHeader {
                    offset: U32::read_from_io(&mut reader)?.into(),
                    num: U32::read_from_io(&mut reader)?,
                }
            }
            Ok((
                Cow::Owned(block_headers),
                &data[usize::try_from(reader.position()).unwrap_or_default()..],
            ))
        }
    }

    /// Returns the data of a block.
    fn get<'data>(&self, data: &'data [u8], block_type: BlockType) -> &'data [u8] {
        let start = usize::try_from(self[block_type].offset()).unwrap();
        let end = self
            .iter()
            .skip(block_type as usize + 1)
            .find(|block_header| block_header.offset > U64::ZERO)
            .map_or(start, |block| usize::try_from(block.offset()).unwrap());
        &data[start..end]
    }

    /// Returns the sections block.
    fn sections_block<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        self.get(data, BlockType::Sections)
    }

    pub fn sections<'data>(&self, data: &'data [u8]) -> impl Iterator<Item = &'data Section> {
        let sections = self.sections_block(data);
        let section_size = sections.len() / self[BlockType::Sections].num() as usize;
        sections
            .chunks_exact(section_size)
            .flat_map(Section::ref_from_bytes)
    }

    /// Returns the entries block.
    fn entries_block<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        self.get(data, BlockType::Entries)
    }

    /// Returns a slice of [`Entry`] from the entries block.
    pub fn entries<'data>(&self, data: &'data [u8]) -> Result<&'data [Entry], NsisError> {
        <[Entry]>::try_ref_from_bytes(self.entries_block(data))
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))
    }

    /// Returns the strings block.
    pub fn strings_block<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        self.get(data, BlockType::Strings)
    }

    /// Returns the language table block.
    pub fn language_table_block<'data>(&self, data: &'data [u8]) -> &'data [u8] {
        self.get(data, BlockType::LangTables)
    }

    /// Returns an iterator of [`LanguageTable`] in the language table block.
    pub fn language_tables<'data>(
        &self,
        data: &'data [u8],
    ) -> impl Iterator<Item = &'data LanguageTable> {
        let language_table_block = self.language_table_block(data);

        // This should match the language table size defined in the NSIS header, but calculating it
        // manually covers the case when the defined language table size is 0 or -1.
        let language_table_size = language_table_block
            .len()
            .checked_div(self[BlockType::LangTables].num() as usize)
            .unwrap_or(language_table_block.len());

        language_table_block
            .chunks_exact(language_table_size)
            .flat_map(LanguageTable::ref_from_bytes)
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

impl fmt::Debug for BlockHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct NamedBlockHeader {
            r#type: BlockType,
            block_header: BlockHeader,
        }

        impl fmt::Debug for NamedBlockHeader {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(self.r#type.as_str())
                    .field("offset", &self.block_header.offset())
                    .field("num", &self.block_header.num())
                    .finish()
            }
        }

        f.debug_list()
            .entries(
                BlockType::iter()
                    .zip(self.iter())
                    .map(|(r#type, &block_header)| NamedBlockHeader {
                        r#type,
                        block_header,
                    }),
            )
            .finish()
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
