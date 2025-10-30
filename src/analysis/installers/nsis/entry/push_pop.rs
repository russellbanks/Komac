use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum PushPop {
    Push = 0u32,
    Pop = 1u32.to_le(),
}

impl PushPop {
    /// Returns `true` if this is a push instruction.
    pub const fn is_push(self) -> bool {
        matches!(self, Self::Push)
    }

    /// Returns `true` if this is a pop instruction.
    #[inline]
    pub const fn is_pop(self) -> bool {
        matches!(self, Self::Pop)
    }
}
