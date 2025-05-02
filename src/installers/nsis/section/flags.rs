use std::fmt;

use bitflags::{Bits, bitflags};
use derive_more::Deref;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Copy, Clone, Deref, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct SectionFlags(i32);

bitflags! {
    impl SectionFlags: i32 {
        const SELECTED = 1i32.to_le();
        const SECTION_GROUP = (1i32 << 1).to_le();
        const SECTION_GROUP_END = (1i32 << 2).to_le();
        const BOLD = (1i32 << 3).to_le();
        const RO = (1i32 << 4).to_le();
        const EXPAND = (1i32 << 5).to_le();
        const PSELECTED = (1i32 << 6).to_le();
        const TOGGLED = (1i32 << 7).to_le();
        const NAME_CHANGE = (1i32 << 8).to_le();
    }
}

impl fmt::Debug for SectionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_fmt(format_args!("{:#x}", <i32 as Bits>::EMPTY))
        } else {
            fmt::Display::fmt(self, f)
        }
    }
}

impl fmt::Display for SectionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        bitflags::parser::to_writer(&Self(**self), f)
    }
}
