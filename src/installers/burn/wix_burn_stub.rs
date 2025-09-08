use std::ops::Range;

use zerocopy::{Immutable, KnownLayout, TryFromBytes, little_endian::U32};

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum WixBurnStubMagic {
    F14300 = 0x00F1_4300_u32.to_le(),
}

/// <https://github.com/wixtoolset/wix/blob/main/src/burn/stub/StubSection.cpp>
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct WixBurnStub {
    magic: WixBurnStubMagic,
    version: U32,
    guid: uuid::Bytes,
    stub_size: U32,
    original_checksum: U32,
    original_signature_offset: U32,
    original_signature_size: U32,
    container_format: U32,
    container_count: U32,
    bootstrapper_application_container_size: U32,
    // (512 (minimum section size) - 52 (size of above data)) / 4 (size of DWORD)
    attached_container_sizes: [U32; 115],
}

impl WixBurnStub {
    pub const fn ux_container_slice_range(&self) -> Range<usize> {
        let stub_size = self.stub_size.get() as usize;
        stub_size..stub_size + self.bootstrapper_application_container_size.get() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::WixBurnStub;

    #[test]
    fn wix_burn_stub_size() {
        const MINIMUM_PE_SECTION_SIZE: usize = 512;

        assert_eq!(size_of::<WixBurnStub>(), MINIMUM_PE_SECTION_SIZE)
    }
}
