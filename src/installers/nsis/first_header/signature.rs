use std::fmt::{Debug, Formatter};
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[repr(u32)]
enum Magic {
    /// Default NSIS magic bytes
    DeadBeef = 0xDEAD_BEEF,
    /// Present in NSIS 1.1e..<1.30
    DeadBeed = 0xDEAD_BEED,
}

/// NSIS 1.00 signature `nsisinstall\0` with `0xDEAD_BEEF` magic bytes
const DEAD_BEEF_NSIS_INSTALL: [U32; 4] = [
    U32::new(Magic::DeadBeef as u32),
    U32::new(u32::from_le_bytes(*b"nsis")),
    U32::new(u32::from_le_bytes(*b"inst")),
    U32::new(u32::from_le_bytes(*b"all\0")),
];

/// NSIS 1.1e Signature `NullSoftInst` with `0xDEAD_BEED` magic bytes
const DEAD_BEED_NULLSOFT_U: [U32; 4] = [
    U32::new(Magic::DeadBeed as u32),
    U32::new(u32::from_le_bytes(*b"Null")),
    U32::new(u32::from_le_bytes(*b"Soft")),
    U32::new(u32::from_le_bytes(*b"Inst")),
];

/// NSIS 1.30 Signature `NullSoftInst` with `0xDEAD_BEEF` magic bytes
const DEAD_BEEF_NULLSOFT_U: [U32; 4] = [
    U32::new(Magic::DeadBeef as u32),
    U32::new(u32::from_le_bytes(*b"Null")),
    U32::new(u32::from_le_bytes(*b"Soft")),
    U32::new(u32::from_le_bytes(*b"Inst")),
];

/// NSIS 1.60b2+ Signature `NullsoftInst` with `0xDEAD_BEEF` magic bytes
const DEAD_BEEF_NULLSOFT_L: [U32; 4] = [
    U32::new(Magic::DeadBeef as u32),
    U32::new(u32::from_le_bytes(*b"Null")),
    U32::new(u32::from_le_bytes(*b"soft")),
    U32::new(u32::from_le_bytes(*b"Inst")),
];

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct NsisSignature([U32; 4]);

impl NsisSignature {
    pub const fn is_valid(&self) -> bool {
        matches!(
            self.0,
            DEAD_BEEF_NULLSOFT_L
                | DEAD_BEEF_NULLSOFT_U
                | DEAD_BEED_NULLSOFT_U
                | DEAD_BEEF_NSIS_INSTALL
        )
    }
}

impl Debug for NsisSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for section in &self.0 {
            if let Ok(str) = std::str::from_utf8(section.as_bytes()) {
                write!(f, "{str}")?;
            } else {
                write!(f, "{:08x}", section.get())?;
            }
        }
        Ok(())
    }
}
