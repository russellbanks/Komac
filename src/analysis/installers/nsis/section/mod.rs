mod flags;

use std::fmt;

use flags::SectionFlags;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LittleEndian};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Section {
    pub name: I32<LittleEndian>,
    pub install_types: I32<LittleEndian>,
    pub flags: SectionFlags,
    pub code: I32<LittleEndian>,
    pub code_size: I32<LittleEndian>,
    pub size_kb: I32<LittleEndian>,
    pub rest: [u8],
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
