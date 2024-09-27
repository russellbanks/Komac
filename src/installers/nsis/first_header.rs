use bitflags::bitflags;
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, KnownLayout, TryFromBytes};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
struct HeaderFlags(u32);

bitflags! {
    impl HeaderFlags: u32 {
        const UNINSTALL = 1 << 0;
        const SILENT = 1 << 1;
        const NO_CRC = 1 << 2;
        const FORCE_CRC = 1 << 3;
        // NSISBI fork flags:
        const BI_LONG_OFFSET = 1 << 4;
        const BI_EXTERNAL_FILE_SUPPORT = 1 << 5;
        const BI_EXTERNAL_FILE = 1 << 6;
        const BI_IS_STUB_INSTALLER = 1 << 7;
    }
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum Magic1 {
    DeadBeef = 0xDEAD_BEEF,
    DeadBeed = 0xDEAD_BEED,
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum Magic2 {
    Null = u32::from_le_bytes(*b"Null"),
    Nsis = u32::from_le_bytes(*b"nsis"),
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum Magic3 {
    SoftL = u32::from_le_bytes(*b"soft"),
    SoftU = u32::from_le_bytes(*b"Soft"),
    Inst = u32::from_le_bytes(*b"inst"),
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum Magic4 {
    Inst = u32::from_le_bytes(*b"Inst"),
    All0 = u32::from_le_bytes(*b"all\0"),
}

#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
struct NsisSignature(Magic1, Magic2, Magic3, Magic4);

#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FirstHeader {
    flags: HeaderFlags,
    signature: NsisSignature,
    pub header_size: U32,
    length_of_following_data: U32,
}

impl FirstHeader {
    /// The NSIS first header is aligned to 512 bytes
    pub const ALIGNMENT: u16 = 512;
}
