use std::{
    fmt, io,
    io::{Read, Seek},
};

use itertools::Itertools;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U32};

use super::super::{SectionReader, SectionTable};

/// Represents a PE data directory.
///
/// See <https://learn.microsoft.com/windows/win32/api/winnt/ns-winnt-image_data_directory>.
#[derive(Copy, Clone, Eq, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct DataDirectory {
    virtual_address: U32<LittleEndian>,
    size: U32<LittleEndian>,
}

impl DataDirectory {
    /// Creates a new [`DataDirectory`] from a virtual address and a size.
    #[inline]
    pub const fn new(virtual_address: u32, size: u32) -> Self {
        Self {
            virtual_address: U32::new(virtual_address),
            size: U32::new(size),
        }
    }

    /// Returns the relative virtual address of the data directory table.
    #[inline]
    pub const fn virtual_address(self) -> u32 {
        self.virtual_address.get()
    }

    /// Returns the size of the data directory table, in bytes.
    #[inline]
    pub const fn size(self) -> u32 {
        self.size.get()
    }

    /// Returns the file offset of the data directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory's virtual address was not found in any section in the
    /// section table.
    #[inline]
    pub fn file_offset(self, section_table: &SectionTable) -> io::Result<u32> {
        section_table.to_file_offset(self.virtual_address())
    }

    pub fn section_reader<R: Read + Seek>(
        self,
        mut reader: R,
        section_table: &SectionTable,
    ) -> io::Result<SectionReader<R>> {
        let mut directory_offset = self.file_offset(section_table)?;

        SectionReader::new(reader, directory_offset.into(), self.size().into())
    }
}

impl fmt::Debug for DataDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IMAGE_DATA_DIRECTORY")
            .field("VirtualAddress", &self.virtual_address())
            .field("Size", &self.size())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::DataDirectory;

    #[test]
    fn size() {
        assert_eq!(size_of::<DataDirectory>(), size_of::<u64>());
    }
}
