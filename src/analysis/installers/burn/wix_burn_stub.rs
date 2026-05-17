use std::{fmt, io, ops::Range};

use uuid::Uuid;
use zerocopy::{
    Immutable, IntoBytes, KnownLayout, TryFromBytes, little_endian::U32, transmute_ref,
};

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/burn/engine/inc/engine.h#L9>
#[expect(dead_code)]
#[derive(Copy, Clone, TryFromBytes, KnownLayout, Immutable, IntoBytes)]
#[repr(u32)]
enum WixBurnStubMagic {
    F14300 = 0x00F1_4300_u32.to_le(),
}

impl fmt::Debug for WixBurnStubMagic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}

impl fmt::UpperHex for WixBurnStubMagic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <u32 as fmt::UpperHex>::fmt(transmute_ref!(self), f)
    }
}

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/burn/stub/StubSection.cpp>
#[derive(TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct WixBurnStub {
    /// 0x00F14300
    magic: WixBurnStubMagic,

    /// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/burn/engine/inc/engine.h#L10
    version: U32,

    guid: uuid::Bytes,

    /// Returns the size of the stub.
    stub_size: U32,

    original_checksum: U32,

    original_signature_offset: U32,

    original_signature_size: U32,

    /// 1 = CAB
    container_type: U32,

    container_count: U32,

    /// Byte count of manifest + UX container
    bootstrapper_application_container_size: U32,

    // (512 (minimum section size) - 52 (size of above data)) / 4 (size of DWORD)
    attached_container_sizes: [U32; 115],
}

impl WixBurnStub {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    /// Returns the version.
    #[inline]
    pub const fn version(&self) -> u32 {
        self.version.get()
    }

    /// Returns the bundle GUID.
    #[inline]
    pub const fn bundle_guid(&self) -> Uuid {
        Uuid::from_bytes(self.guid)
    }

    /// Returns the stub size.
    #[inline]
    pub const fn stub_size(&self) -> u32 {
        self.stub_size.get()
    }

    #[inline]
    pub const fn original_checksum(&self) -> u32 {
        self.original_checksum.get()
    }

    #[inline]
    pub const fn original_signature_offset(&self) -> u32 {
        self.original_signature_offset.get()
    }

    #[inline]
    pub const fn original_signature_size(&self) -> u32 {
        self.original_signature_size.get()
    }

    #[inline]
    pub const fn container_type(&self) -> u32 {
        self.container_type.get()
    }

    /// Returns the number of containers.
    #[inline]
    pub const fn container_count(&self) -> u32 {
        self.container_count.get()
    }

    #[inline]
    pub const fn bootstrapper_application_container_size(&self) -> u32 {
        self.bootstrapper_application_container_size.get()
    }

    #[expect(unused)]
    pub const fn ux_container_range(&self) -> Range<u32> {
        let stub_size = self.stub_size();
        stub_size..stub_size + self.bootstrapper_application_container_size()
    }
}

impl fmt::Debug for WixBurnStub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WixBurnStub")
            .field("dwMagic", &self.magic)
            .field("dwVersion", &self.version())
            .field(
                "guidBundleCode",
                &self
                    .bundle_guid()
                    .as_braced()
                    .encode_upper(&mut Uuid::encode_buffer()),
            )
            .field("dwStubSize", &self.stub_size())
            .field("dwOriginalChecksum", &self.original_checksum())
            .field(
                "dwOriginalSignatureOffset",
                &self.original_signature_offset(),
            )
            .field("dwOriginalSignatureSize", &self.original_signature_size())
            .field("dwContainerFormat", &self.container_type())
            .field("dwContainerCount", &self.container_count())
            .field(
                "bootstrapperApplicationContainerSize",
                &self.bootstrapper_application_container_size(),
            )
            .field(
                "qwAttachedContainerSizes",
                &self.attached_container_sizes.map(U32::get),
            )
            .finish()
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

    #[test]
    fn wix_burn_stub_alignment() {
        assert_eq!(align_of::<WixBurnStub>(), align_of::<u32>())
    }
}
