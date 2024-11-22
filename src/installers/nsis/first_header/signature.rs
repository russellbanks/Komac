use std::fmt::Debug;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Magic {
    /// Default NSIS magic bytes
    DeadBeef = 0xDEAD_BEEFu32.to_le(),
    /// Present in NSIS 1.1e..<1.30
    DeadBeed = 0xDEAD_BEEDu32.to_le(),
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig1 {
    Null = u32::from_le_bytes(*b"Null").to_le(),
    Nsis = u32::from_le_bytes(*b"nsis").to_le(),
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig2 {
    SoftU = u32::from_le_bytes(*b"Soft").to_le(),
    SoftL = u32::from_le_bytes(*b"soft").to_le(),
    Inst = u32::from_le_bytes(*b"inst").to_le(),
}

#[expect(dead_code)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Sig3 {
    Inst = u32::from_le_bytes(*b"Inst").to_le(),
    All0 = u32::from_le_bytes(*b"all\0").to_le(),
}

#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct NsisSignature(Sig1, Sig2, Sig3);
