mod flags;
mod signature;

use derive_more::Debug;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, little_endian::U32};

use crate::installers::nsis::first_header::{
    flags::HeaderFlags,
    signature::{Magic, NsisSignature},
};

#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FirstHeader {
    flags: HeaderFlags,
    magic: Magic,
    signature: NsisSignature,
    #[debug("{length_of_header}")]
    pub length_of_header: U32,
    #[debug("{length_of_following_data}")]
    length_of_following_data: U32,
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
