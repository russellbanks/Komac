use crate::installers::nsis::entry::which::WhichEntry;
use zerocopy::little_endian::U32;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

pub mod registry;
pub mod which;

const MAX_ENTRY_OFFSETS: usize = 6;

#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Entry {
    pub which: WhichEntry,
    pub offsets: [U32; MAX_ENTRY_OFFSETS],
}
