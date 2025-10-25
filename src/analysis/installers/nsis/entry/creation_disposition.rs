use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-createfilea>
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum CreationDisposition {
    /// Creates a new file, only if it does not already exist.
    ///
    /// If the specified file exists, the function fails and the last-error code is set to
    /// **ERROR_FILE_EXISTS** (80).
    ///
    /// If the specified file does not exist and is a valid path to a writable location, a new file
    /// is created.
    #[doc(alias = "CREATE_NEW")]
    CreateNew = 1,
    /// Creates a new file, always.
    ///
    /// If the specified file exists and is writable, the function truncates the file, the function
    /// succeeds, and last-error code is set to **ERROR_ALREADY_EXISTS** (183).
    ///
    /// If the specified file does not exist and is a valid path, a new file is created, the
    /// function succeeds, and the last-error code is set to zero.
    #[doc(alias = "CREATE_ALWAYS")]
    CreateAlways = 2,
    /// Opens a file or device, only if it exists.
    ///
    /// If the specified file or device does not exist, the function fails and the last-error code
    /// is set to **ERROR_FILE_NOT_FOUND** (2).
    #[doc(alias = "OPEN_EXISTING")]
    OpenExisting = 3,
    /// Opens a file, always.
    ///
    /// If the specified file exists, the function succeeds and the last-error code is set to
    /// **ERROR_ALREADY_EXISTS** (183).
    ///
    /// If the specified file does not exist and is a valid path to a writable location, the
    /// function creates a file and the last-error code is set to zero.
    #[doc(alias = "OPEN_ALWAYS")]
    OpenAlways = 4,
    /// Opens a file and truncates it so that its size is zero bytes, only if it exists.
    /// If the specified file does not exist, the function fails and the last-error code is set to
    /// **ERROR_FILE_NOT_FOUND** (2).
    ///
    /// The calling process must open the file with the **GENERIC_WRITE** bit set as part of the
    /// *dwDesiredAccess* parameter.
    #[doc(alias = "TRUNCATE_EXISTING")]
    TruncateExisting = 5,
}

impl CreationDisposition {
    /// Returns the Creation Disposition as a static string in screaming snake case.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateNew => "CREATE_NEW",
            Self::CreateAlways => "CREATE_ALWAYS",
            Self::OpenExisting => "OPEN_EXISTING",
            Self::OpenAlways => "OPEN_ALWAYS",
            Self::TruncateExisting => "TRUNCATE_EXISTING",
        }
    }
}

impl fmt::Display for CreationDisposition {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<CreationDisposition> for u32 {
    #[inline]
    fn from(creation_disposition: CreationDisposition) -> Self {
        creation_disposition as Self
    }
}
