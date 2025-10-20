use std::fmt;

use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/fileform.h#L527>
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, KnownLayout, Immutable,
)]
#[repr(transparent)]
pub struct DelFlags(u32);

bitflags! {
    impl DelFlags: u32 {
        const DIRECTORY = 1u32.to_le();
        const RECURSE = (1u32 << 1).to_le();
        const REBOOT = (1u32 << 2).to_le();
        const SIMPLE = (1u32 << 3).to_le();
    }
}

impl fmt::Display for DelFlags {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}
