use std::{io, io::Read};

use zerocopy::{Immutable, KnownLayout, LittleEndian, TryFromBytes, U32};

/// Standard 64-bit COFF fields (for `PE32+`).
///
/// In `winnt.h`, this is a subset of [`IMAGE_OPTIONAL_HEADER64`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header64).
///
/// * For 32-bit version, see [`StandardFields32`].
/// * For unified version, see [`StandardFields`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct StandardFields64 {
    /// See docs for [`StandardFields::magic`](crate::pe::optional_header::StandardFields::magic).
    #[doc(alias = "Magic")]
    magic: Magic,
    /// See docs for [`StandardFields::major_linker_version`].
    #[doc(alias = "MajorLinkerVersion")]
    major_linker_version: u8,
    /// See docs for [`StandardFields::minor_linker_version`].
    #[doc(alias = "MinorLinkerVersion")]
    minor_linker_version: u8,
    /// See docs for [`StandardFields::size_of_code`].
    #[doc(alias = "SizeOfCode")]
    size_of_code: U32<LittleEndian>,
    /// See docs for [`StandardFields::size_of_initialized_data`].
    #[doc(alias = "SizeOfInitializedData")]
    size_of_initialized_data: U32<LittleEndian>,
    /// See docs for [`StandardFields::size_of_uninitialized_data`].
    #[doc(alias = "SizeOfUninitializedData")]
    size_of_uninitialized_data: U32<LittleEndian>,
    /// See docs for [`StandardFields::address_of_entry_point`].
    address_of_entry_point: U32<LittleEndian>,
    /// See docs for [`StandardFields::base_of_code`].
    base_of_code: U32<LittleEndian>,
}

impl StandardFields64 {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    #[inline]
    pub const fn major_linker_version(&self) -> u8 {
        self.major_linker_version
    }

    #[inline]
    pub const fn minor_linker_version(&self) -> u8 {
        self.minor_linker_version
    }

    #[inline]
    pub const fn size_of_code(&self) -> u32 {
        self.size_of_code.get()
    }

    #[inline]
    pub const fn size_of_initialized_data(&self) -> u32 {
        self.size_of_initialized_data.get()
    }

    #[inline]
    pub const fn size_of_uninitialized_data(&self) -> u32 {
        self.size_of_uninitialized_data.get()
    }

    #[inline]
    pub const fn address_of_entry_point(&self) -> u32 {
        self.address_of_entry_point.get()
    }

    #[inline]
    pub const fn base_of_code(&self) -> u32 {
        self.base_of_code.get()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromBytes, Immutable, KnownLayout)]
#[repr(u16)]
pub enum Magic {
    #[doc(alias = "IMAGE_NT_OPTIONAL_HDR64_MAGIC")]
    ImageNtOptionalHdr64 = 0x20b_u16.to_le(),
}
