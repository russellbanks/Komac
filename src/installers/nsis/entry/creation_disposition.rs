use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-createfilea>
#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum CreationDisposition {
    CreateNew = 1,
    CreateAlways = 2,
    OpenExisting = 3,
    OpenAlways = 4,
    TruncateExisting = 5,
}

impl CreationDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateNew => "CreateNew",
            Self::CreateAlways => "CreateAlways",
            Self::OpenExisting => "OpenExisting",
            Self::OpenAlways => "OpenAlways",
            Self::TruncateExisting => "TruncateExisting",
        }
    }
}

impl fmt::Display for CreationDisposition {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
