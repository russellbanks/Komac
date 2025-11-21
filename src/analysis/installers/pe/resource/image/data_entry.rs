use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U32};

// Each resource data entry describes a leaf node in the resource directory tree. It contains an
// offset, relative to the beginning of the resource directory of the data for the resource, a size
// field that gives the number of bytes of data at that offset, a CodePage that should be used when
// decoding code point values within the resource data. Typically, for new applications the code
// page would be the Unicode code page.

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct ImageResourceDataEntry {
    /// RVA of the data.
    offset_to_data: U32<LittleEndian>,
    size: U32<LittleEndian>,
    codepage: U32<LittleEndian>,
    reserved: U32<LittleEndian>,
}

impl ImageResourceDataEntry {
    #[inline]
    pub const fn offset_to_data(&self) -> u32 {
        self.offset_to_data.get()
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.size.get()
    }

    #[inline]
    pub const fn codepage(&self) -> u32 {
        self.codepage.get()
    }
}
