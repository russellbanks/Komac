use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(i32)]
pub enum PushPop {
    Push = 0i32.to_le(),
    Pop = 1i32.to_le(),
}
