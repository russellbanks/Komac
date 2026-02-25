use std::fmt;

use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A series of bitflags representing the file flags for the [`VS_FIXEDFILEINFO`](https://docs.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo)
/// structure.
#[repr(C)]
#[derive(Copy, Clone, FromBytes, KnownLayout, Immutable)]
pub struct VSFileFlags(u32);

bitflags! {
    impl VSFileFlags: u32 {
        const DEBUG = 1u32.to_le();
        const PRERELEASE = (1u32 << 1).to_le();
        const PATCHED = (1u32 << 2).to_le();
        const PRIVATEBUILD = (1u32 << 3).to_le();
        const INFOINFERRED = (1u32 << 4).to_le();
        const SPECIALBUILD = (1u32 << 5).to_le();
    }
}

impl fmt::Debug for VSFileFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{:#x}", Self::empty().bits())
        } else {
            bitflags::parser::to_writer(self, f)
        }
    }
}
