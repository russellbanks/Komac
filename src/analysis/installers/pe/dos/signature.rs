use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, TryFromBytes, IntoBytes, KnownLayout, Immutable,
)]
#[repr(u16)]
pub enum DosSignature {
    #[default]
    MZ = u16::from_le_bytes(*b"MZ"),
}
