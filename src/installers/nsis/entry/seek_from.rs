use derive_more::Display;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-showwindow>
#[expect(dead_code)]
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum SeekFrom {
    Set,
    Current,
    End,
}
