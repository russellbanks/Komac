use std::fmt;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U16, U32};

#[derive(Copy, Clone, Eq, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct SectionHeader {
    name: [u8; 8],
    virtual_size: U32<LittleEndian>,
    virtual_address: U32<LittleEndian>,
    size_of_raw_data: U32<LittleEndian>,
    pointer_to_raw_data: U32<LittleEndian>,
    pointer_to_relocations: U32<LittleEndian>,
    pointer_to_line_numbers: U32<LittleEndian>,
    number_of_relocations: U16<LittleEndian>,
    number_of_line_numbers: U16<LittleEndian>,
    characteristics: U32<LittleEndian>,
}

impl SectionHeader {
    pub fn real_name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .map(|name| name.trim_end_matches('\0'))
            .unwrap_or_default()
    }

    #[inline]
    pub const fn raw_name(&self) -> [u8; 8] {
        self.name
    }

    #[inline]
    pub const fn virtual_size(&self) -> u32 {
        self.virtual_size.get()
    }

    #[inline]
    pub const fn virtual_address(&self) -> u32 {
        self.virtual_address.get()
    }

    #[inline]
    pub const fn size_of_raw_data(&self) -> u32 {
        self.size_of_raw_data.get()
    }

    #[inline]
    pub const fn pointer_to_raw_data(&self) -> u32 {
        self.pointer_to_raw_data.get()
    }

    #[inline]
    pub const fn pointer_to_relocations(&self) -> u32 {
        self.pointer_to_relocations.get()
    }

    #[inline]
    pub const fn pointer_to_line_numbers(&self) -> u32 {
        self.pointer_to_line_numbers.get()
    }

    #[inline]
    pub const fn number_of_relocations(&self) -> u16 {
        self.number_of_relocations.get()
    }

    #[inline]
    pub const fn number_of_line_numbers(&self) -> u16 {
        self.number_of_line_numbers.get()
    }

    #[inline]
    pub const fn characteristics(&self) -> u32 {
        self.characteristics.get()
    }
}

impl fmt::Debug for SectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("Name", &self.real_name())
            .field("Raw name", &self.raw_name())
            .field("VirtualSize", &self.virtual_size())
            .field("VirtualAddress", &self.virtual_address())
            .field("SizeOfRawData", &self.size_of_raw_data())
            .field("PointerToRawData", &self.pointer_to_raw_data())
            .field("PointerToRelocations", &self.pointer_to_relocations())
            .field("PointerToLinenumbers", &self.pointer_to_line_numbers())
            .field("NumberOfRelocations", &self.number_of_relocations())
            .field("NumberOfLinenumbers", &self.number_of_line_numbers())
            .field("Characteristics", &self.characteristics())
            .finish()
    }
}
