use std::io;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U32};

use crate::analysis::installers::pe::SectionTable;

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct DataDirectory {
    virtual_address: U32<LittleEndian>,
    size: U32<LittleEndian>,
}

impl DataDirectory {
    #[inline]
    pub const fn virtual_address(self) -> u32 {
        self.virtual_address.get()
    }

    #[inline]
    pub const fn size(self) -> u32 {
        self.size.get()
    }

    #[inline]
    pub fn file_offset(self, section_table: &SectionTable) -> io::Result<u32> {
        section_table.to_file_offset(self.virtual_address())
    }
}
