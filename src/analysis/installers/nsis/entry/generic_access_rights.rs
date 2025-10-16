use std::fmt;

use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, KnownLayout, Immutable,
)]
#[repr(transparent)]
pub struct GenericAccessRights(u32);

bitflags! {
    impl GenericAccessRights: u32 {
        const GENERIC_ALL = (1u32 << 28).to_le();
        const GENERIC_EXECUTE = (1u32 << 29).to_le();
        const GENERIC_WRITE = (1u32 << 30).to_le();
        const GENERIC_READ = (1u32 << 31).to_le();
    }
}

impl fmt::Display for GenericAccessRights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}
