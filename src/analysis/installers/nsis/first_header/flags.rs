use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct HeaderFlags(u32);

bitflags! {
    impl HeaderFlags: u32 {
        const UNINSTALL = 1u32.to_le();
        const SILENT = (1u32 << 1).to_le();
        const NO_CRC = (1u32 << 2).to_le();
        const FORCE_CRC = (1u32 << 3).to_le();
        // NSISBI fork flags:
        const BI_LONG_OFFSET = (1u32 << 4).to_le();
        const BI_EXTERNAL_FILE_SUPPORT = (1u32 << 5).to_le();
        const BI_EXTERNAL_FILE = (1u32 << 6).to_le();
        const BI_IS_STUB_INSTALLER = (1u32 << 7).to_le();
    }
}
