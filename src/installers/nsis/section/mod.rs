mod flags;

use derive_more::Debug;
use flags::SectionFlags;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LittleEndian};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Section {
    pub name: I32<LittleEndian>,
    pub install_types: I32<LittleEndian>,
    pub flags: SectionFlags,
    pub code: I32<LittleEndian>,
    pub code_size: I32<LittleEndian>,
    pub size_kb: I32<LittleEndian>,
    #[debug(skip)]
    pub rest: [u8],
}
