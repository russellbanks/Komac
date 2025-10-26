mod flags;

use std::fmt;

use flags::SectionFlags;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LE};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Section {
    name: I32<LE>,
    install_types: I32<LE>,
    flags: SectionFlags,
    code: I32<LE>,
    code_size: I32<LE>,
    size_kb: I32<LE>,
    rest: [u8],
}

impl Section {
    /// Returns an offset to the name in the string table.
    #[inline]
    pub const fn name_offset(&self) -> i32 {
        self.name.get()
    }

    #[inline]
    pub const fn code_offset(&self) -> i32 {
        self.code.get()
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("Name", &self.name)
            .field("InstallTypes", &self.install_types)
            .field("Flags", &self.flags)
            .field("Code", &self.code)
            .field("Code size", &self.code_size)
            .field("Size KB", &self.size_kb)
            .finish_non_exhaustive()
    }
}
