use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Copy, Clone, Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct CommonHeaderFlags(u32);

bitflags! {
    impl CommonHeaderFlags: u32 {
        const DETAILS_SHOWDETAILS = 1u32.to_le();
        const DETAILS_NEVERSHOW = (1u32 << 1).to_le();
        const PROGRESS_COLORED = (1u32 << 2).to_le();
        const FORCE_CRC = (1u32 << 3).to_le();
        const SILENT = (1u32 << 4).to_le();
        const SILENT_LOG = (1u32 << 5).to_le();
        const AUTO_CLOSE = (1u32 << 6).to_le();
        const DIR_NO_SHOW = (1u32 << 7).to_le();
        const NO_ROOT_DIR = (1u32 << 8).to_le();
        const COMP_ONLY_ON_CUSTOM = (1u32 << 9).to_le();
        const NO_CUSTOM = (1u32 << 10).to_le();
    }
}
