use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum PushPop {
    Push = 0u32,
    Pop = 1u32.to_le(),
}
