use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-showwindow>
#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum SeekFrom {
    Set,
    Current,
    End,
}

impl SeekFrom {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Set => "Set",
            Self::Current => "Current",
            Self::End => "End",
        }
    }
}

impl fmt::Display for SeekFrom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
