use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct CommonHeaderFlags(U32);

#[expect(dead_code)]
impl CommonHeaderFlags {
    pub const DETAILS_SHOWDETAILS: Self = Self(U32::new(1 << 0));
    pub const DETAILS_NEVERSHOW: Self = Self(U32::new(1 << 1));
    pub const PROGRESS_COLORED: Self = Self(U32::new(1 << 2));
    pub const SILENT: Self = Self(U32::new(1 << 3));
    pub const SILENT_LOG: Self = Self(U32::new(1 << 4));
    pub const AUTO_CLOSE: Self = Self(U32::new(1 << 5));
    pub const DIR_NO_SHOW: Self = Self(U32::new(1 << 6));
    pub const NO_ROOT_DIR: Self = Self(U32::new(1 << 7));
    pub const COMP_ONLY_ON_CUSTOM: Self = Self(U32::new(1 << 7));
    pub const NO_CUSTOM: Self = Self(U32::new(1 << 7));
}
