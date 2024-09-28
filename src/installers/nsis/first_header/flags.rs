use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct HeaderFlags(U32);

#[expect(dead_code)]
impl HeaderFlags {
    pub const UNINSTALL: Self = Self(U32::new(1 << 0));
    pub const SILENT: Self = Self(U32::new(1 << 1));
    pub const NO_CRC: Self = Self(U32::new(1 << 2));
    pub const FORCE_CRC: Self = Self(U32::new(1 << 3));
    // NSISBI fork flags:
    pub const BI_LONG_OFFSET: Self = Self(U32::new(1 << 4));
    pub const BI_EXTERNAL_FILE_SUPPORT: Self = Self(U32::new(1 << 5));
    pub const BI_EXTERNAL_FILE: Self = Self(U32::new(1 << 6));
    pub const BI_IS_STUB_INSTALLER: Self = Self(U32::new(1 << 7));
}
