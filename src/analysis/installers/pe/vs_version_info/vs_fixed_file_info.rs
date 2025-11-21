use std::fmt;

use itertools::Itertools;
use nt_time::FileTime;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes, U32, transmute};

use super::VSFileFlags;

/// Represents a [`VS_FIXEDFILEINFO`](https://docs.microsoft.com/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo) structure.
#[repr(C)]
#[derive(Copy, Clone, TryFromBytes, Immutable, KnownLayout)]
pub struct VSFixedFileInfo {
    /// Contains the value `0xFEEF04BD`. This is used with the szKey member of the
    /// [`VS_VERSIONINFO`] structure when searching a file for the VS_FIXEDFILEINFO structure.
    ///
    /// [`VS_VERSIONINFO`]: super::VSVersionInfo
    #[doc(alias = "dwSignature")]
    signature: VSFixedFileInfoSignature,

    /// The binary version number of this structure. The high-order word of this member contains the
    /// major version number, and the low-order word contains the minor version number.
    #[doc(alias = "dwStrucVersion")]
    struct_version: U32<LE>,

    /// The most significant 32 bits of the file's binary version number. This member is used with
    /// `file_version_ls` to form a 64-bit value used for numeric comparisons.
    #[doc(alias = "dwFileVersionMS")]
    file_version_ms: U32<LE>,

    /// The least significant 32 bits of the file's binary version number. This member is used with
    /// `file_version_ms` to form a 64-bit value used for numeric comparisons.
    #[doc(alias = "dwFileVersionLS")]
    file_version_ls: U32<LE>,

    /// The most significant 32 bits of the binary version number of the product with which this
    /// file was distributed. This member is used with `product_version_ls` to form a 64-bit value
    /// used for numeric comparisons.
    #[doc(alias = "dwProductVersionMS")]
    product_version_ms: U32<LE>,

    /// The least significant 32 bits of the binary version number of the product with which this
    /// file was distributed. This member is used with `product_version_ms` to form a 64-bit value
    /// used for numeric comparisons.
    #[doc(alias = "dwProductVersionLS")]
    product_version_ls: U32<LE>,

    /// Contains a bitmask that specifies the valid bits in dwFileFlags.
    file_flags_mask: VSFileFlags,

    /// Contains a bitmask that specifies the Boolean attributes of the file.
    file_flags: VSFileFlags,

    /// The operating system for which this file was designed.
    #[doc(alias = "dwFileOS")]
    file_os: U32<LE>,

    /// The general type of file.
    #[doc(alias = "dwFileType")]
    file_type: U32<LE>,

    /// The function of the file. The possible values depend on the value of `file_type`.
    #[doc(alias = "dwFileSubType")]
    file_subtype: U32<LE>,

    /// The most significant 32 bits of the file's 64-bit binary creation date and time stamp.
    #[doc(alias = "dwFileDateMS")]
    file_date_ms: U32<LE>,

    /// The least significant 32 bits of the file's 64-bit binary creation date and time stamp.
    #[doc(alias = "dwFileDateLS")]
    file_date_ls: U32<LE>,
}

impl VSFixedFileInfo {
    #[expect(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn struct_version(&self) -> (u16, u16) {
        let version = self.struct_version.get();
        let major = (version >> u16::BITS) as u16;
        let minor = version as u16;
        (major, minor)
    }

    pub fn file_version_raw(&self) -> (u16, u16, u16, u16) {
        (
            (self.file_version_ms >> u16::BITS).get() as u16,
            self.file_version_ms.get() as u16,
            (self.file_version_ls >> u16::BITS).get() as u16,
            self.file_version_ls.get() as u16,
        )
    }

    pub fn product_version_raw(&self) -> (u16, u16, u16, u16) {
        (
            (self.product_version_ms >> u16::BITS).get() as u16,
            self.product_version_ms.get() as u16,
            (self.product_version_ls >> u16::BITS).get() as u16,
            self.product_version_ls.get() as u16,
        )
    }

    /// Returns the file flags, with the file flags mask applied.
    pub fn file_flags(&self) -> VSFileFlags {
        self.file_flags & self.file_flags_mask
    }

    pub const fn file_os(&self) -> u32 {
        self.file_os.get()
    }

    pub const fn file_type(&self) -> u32 {
        self.file_type.get()
    }

    pub const fn file_subtype(&self) -> u32 {
        self.file_subtype.get()
    }

    pub const fn file_date(&self) -> FileTime {
        FileTime::from_high_low(self.file_date_ms.get(), self.file_date_ls.get())
    }
}

impl fmt::Debug for VSFixedFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (struct_major, struct_minor) = self.struct_version();
        let (file_major, file_minor, file_patch, file_build) = self.file_version_raw();
        let (product_major, product_minor, product_patch, product_build) =
            self.product_version_raw();

        f.debug_struct("VS_FIXEDFILEINFO")
            .field("dwSignature", &self.signature)
            .field(
                "dwStructVersion",
                &format_args!("{struct_major}.{struct_minor}"),
            )
            .field(
                "dwFileVersion",
                &format_args!("{file_major}.{file_minor}.{file_patch}.{file_build}"),
            )
            .field(
                "dwProductVersion",
                &format_args!("{product_major}.{product_minor}.{product_patch}.{product_build}"),
            )
            .field("dwFileFlagsMask", &self.file_flags_mask)
            .field("dwFileFlags", &self.file_flags)
            .field("dwFileOS", &self.file_os())
            .field("dwFileType", &self.file_type())
            .field("dwFileSubtype", &self.file_subtype())
            .field("dwFileDate", &self.file_date())
            .finish()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(u32)]
pub enum VSFixedFileInfoSignature {
    #[default]
    FEEF04BD = 0xFEEF_04BD_u32.to_le(),
}
