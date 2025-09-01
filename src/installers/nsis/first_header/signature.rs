use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Magic {
    /// Default NSIS magic bytes
    DeadBeef = 0xDEAD_BEEFu32.to_le(),
    /// Present in NSIS 1.1e..<1.30
    DeadBeed = 0xDEAD_BEEDu32.to_le(),
}

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig1 {
    Null = u32::from_le_bytes(*b"Null").to_le(),
    Nsis = u32::from_le_bytes(*b"nsis").to_le(),
}

impl fmt::Display for Sig1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("Null"),
            Self::Nsis => f.write_str("nsis"),
        }
    }
}

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig2 {
    SoftU = u32::from_le_bytes(*b"Soft").to_le(),
    SoftL = u32::from_le_bytes(*b"soft").to_le(),
    Inst = u32::from_le_bytes(*b"inst").to_le(),
}

impl fmt::Display for Sig2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SoftU => f.write_str("Soft"),
            Self::SoftL => f.write_str("soft"),
            Self::Inst => f.write_str("inst"),
        }
    }
}

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig3 {
    Inst = u32::from_le_bytes(*b"Inst").to_le(),
    All0 = u32::from_le_bytes(*b"all\0").to_le(),
}

impl fmt::Display for Sig3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inst => f.write_str("Inst"),
            Self::All0 => f.write_str("all\0"),
        }
    }
}

#[derive(Copy, Clone, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct NsisSignature(Sig1, Sig2, Sig3);

impl fmt::Debug for NsisSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NsisSignature({self})")
    }
}

impl fmt::Display for NsisSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.0, self.1, self.2)
    }
}
