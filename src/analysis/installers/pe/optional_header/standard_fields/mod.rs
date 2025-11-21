mod standard_fields_32;
mod standard_fields_64;

use std::io;

pub use standard_fields_32::StandardFields32;
pub use standard_fields_64::StandardFields64;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// Unified 32/64-bit standard COFF fields (for `PE32` and `PE32+`).
///
/// Notably, a value of this type is a member of [`OptionalHeader`],
/// which in turn represents either
/// * [`IMAGE_OPTIONAL_HEADER32`](https://learn.microsoft.com/windows/win32/api/winnt/ns-winnt-image_optional_header32); or
/// * [`IMAGE_OPTIONAL_HEADER64`](https://learn.microsoft.com/windows/win32/api/winnt/ns-winnt-image_optional_header64)
///
/// from `winnt.h`, depending on the value of [`StandardFields::magic`].
///
/// ## Position in PE binary
///
/// Standard COFF fields are located at the beginning of the [`OptionalHeader`] and before the
/// [`WindowsFields`].
///
/// ## Related structures
///
/// * For 32-bit version, see [`StandardFields32`].
/// * For 64-bit version, see [`StandardFields64`].
///
/// [`OptionalHeader`]: crate::pe::OptionalHeader
///
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StandardFields {
    PE32(StandardFields32),
    PE64(StandardFields64),
}

impl StandardFields {
    /// The major version number of the linker.
    #[inline]
    pub const fn major_linker_version(&self) -> u8 {
        match self {
            Self::PE32(pe32) => pe32.major_linker_version(),
            Self::PE64(pe64) => pe64.major_linker_version(),
        }
    }

    /// The minor version number of the linker.
    #[inline]
    pub const fn minor_linker_version(&self) -> u8 {
        match self {
            Self::PE32(pe32) => pe32.minor_linker_version(),
            Self::PE64(pe64) => pe64.minor_linker_version(),
        }
    }

    /// The size of the code section (.text), in bytes, or the sum of all such sections if there are
    /// multiple code sections.
    #[inline]
    pub const fn size_of_code(&self) -> u32 {
        match self {
            Self::PE32(pe32) => pe32.size_of_code(),
            Self::PE64(pe64) => pe64.size_of_code(),
        }
    }

    /// The size of the initialized data section (.data), in bytes, or the sum of all such sections
    /// if there are multiple initialized data sections.
    #[inline]
    pub const fn size_of_initialized_data(&self) -> u32 {
        match self {
            Self::PE32(pe32) => pe32.size_of_initialized_data(),
            Self::PE64(pe64) => pe64.size_of_initialized_data(),
        }
    }

    /// The size of the uninitialized data section (.bss), in bytes, or the sum of all such sections
    /// if there are multiple uninitialized data sections.
    #[inline]
    pub const fn size_of_uninitialized_data(&self) -> u32 {
        match self {
            Self::PE32(pe32) => pe32.size_of_uninitialized_data(),
            Self::PE64(pe64) => pe64.size_of_uninitialized_data(),
        }
    }

    /// A pointer to the entry point function, relative to the image base address.
    ///
    /// * For executable files, this is the starting address.
    /// * For device drivers, this is the address of the initialization function.
    ///
    /// The entry point function is optional for DLLs. When no entry point is present, this member
    /// is zero.
    #[inline]
    pub const fn address_of_entry_point(&self) -> u32 {
        match self {
            Self::PE32(pe32) => pe32.address_of_entry_point(),
            Self::PE64(pe64) => pe64.address_of_entry_point(),
        }
    }

    /// A pointer to the beginning of the code section (.text), relative to the image base.
    #[inline]
    pub const fn base_of_code(&self) -> u32 {
        match self {
            Self::PE32(pe32) => pe32.base_of_code(),
            Self::PE64(pe64) => pe64.base_of_code(),
        }
    }

    /// A pointer to the beginning of the data section (.data), relative to the image base. Absent
    /// in 64-bit PE32+.
    ///
    /// In other words, it is a Relative virtual address (RVA) of the start of the data (.data)
    /// section when the PE is loaded into memory.
    #[inline]
    pub const fn base_of_data(&self) -> Option<u32> {
        match self {
            Self::PE32(pe32) => Some(pe32.base_of_data()),
            Self::PE64(_) => None,
        }
    }
}

impl From<StandardFields32> for StandardFields {
    #[inline]
    fn from(fields: StandardFields32) -> Self {
        Self::PE32(fields)
    }
}

impl From<StandardFields64> for StandardFields {
    #[inline]
    fn from(fields: StandardFields64) -> Self {
        Self::PE64(fields)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromBytes, Immutable, KnownLayout)]
#[repr(u16)]
pub enum Magic {
    #[doc(alias = "IMAGE_NT_OPTIONAL_HDR32_MAGIC")]
    ImageNtOptionalHdr32 = 0x10b_u16.to_le(),
    #[doc(alias = "IMAGE_NT_OPTIONAL_HDR64_MAGIC")]
    ImageNtOptionalHdr64 = 0x20b_u16.to_le(),
    #[doc(alias = "IMAGE_ROM_OPTIONAL_HDR_MAGIC")]
    ImageRomOptionalHdr = 0x107_u16.to_le(),
}

impl Magic {
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
}

impl From<standard_fields_32::Magic> for Magic {
    fn from(_magic: standard_fields_32::Magic) -> Self {
        Self::ImageNtOptionalHdr32
    }
}

impl From<Magic> for standard_fields_32::Magic {
    fn from(_magic: Magic) -> Self {
        Self::ImageNtOptionalHdr32
    }
}

impl From<standard_fields_64::Magic> for Magic {
    fn from(_magic: standard_fields_64::Magic) -> Self {
        Self::ImageNtOptionalHdr64
    }
}

impl From<Magic> for standard_fields_64::Magic {
    fn from(_magic: Magic) -> Self {
        Self::ImageNtOptionalHdr64
    }
}
