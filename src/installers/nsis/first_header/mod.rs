mod flags;
mod signature;

use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes, little_endian::U32};

use crate::installers::nsis::first_header::{
    flags::HeaderFlags,
    signature::{Magic, NsisSignature},
};

#[derive(Copy, Clone, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FirstHeader {
    flags: HeaderFlags,
    magic: Magic,
    signature: NsisSignature,
    pub length_of_header: U32,
    length_of_following_data: U32,
}

impl fmt::Debug for FirstHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstHeader")
            .field("Flags", &self.flags)
            .field("Magic", &self.magic)
            .field("Signature", &self.signature)
            .field("LengthOfHeader", &self.length_of_header.get())
            .field(
                "LengthOfFollowingData",
                &self.length_of_following_data.get(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::installers::nsis::first_header::FirstHeader;

    #[test]
    fn first_header_size() {
        const EXPECTED_FIRST_HEADER_SIZE: usize = size_of::<u32>() * 7;

        assert_eq!(size_of::<FirstHeader>(), EXPECTED_FIRST_HEADER_SIZE);
    }
}
