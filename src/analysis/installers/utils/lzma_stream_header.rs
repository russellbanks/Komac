use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Clone, Copy, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct LzmaStreamHeader {
    props: u8,
    dict_size: U32<LE>,
}

impl LzmaStreamHeader {
    /// Returns the LZMA properties byte.
    #[must_use]
    #[inline]
    pub const fn props(self) -> u8 {
        self.props
    }

    /// Returns the LZMA dictionary size.
    #[must_use]
    #[inline]
    pub const fn dictionary_size(self) -> u32 {
        self.dict_size.get()
    }
}
