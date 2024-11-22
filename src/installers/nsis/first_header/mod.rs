mod flags;
mod signature;

use crate::installers::nsis::first_header::flags::HeaderFlags;
use crate::installers::nsis::first_header::signature::{Magic, NsisSignature};
use zerocopy::little_endian::U32;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FirstHeader {
    flags: HeaderFlags,
    magic: Magic,
    signature: NsisSignature,
    pub length_of_header: U32,
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
