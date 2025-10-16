use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes, ValidityError, try_transmute};

/// <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-showwindow>
#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum ShowWindow {
    Hide,
    ShowNormal,
    ShowMinimized,
    ShowMaximized,
    ShowNoActivate,
    Show,
    Minimize,
    ShowMinNoActive,
    ShowNA,
    Restore,
    ShowDefault,
    ForceMinimize,
}

impl ShowWindow {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hide => "Hide",
            Self::ShowNormal => "ShowNormal",
            Self::ShowMinimized => "ShowMinimized",
            Self::ShowMaximized => "ShowMaximized",
            Self::ShowNoActivate => "ShowNoActivate",
            Self::Show => "Show",
            Self::Minimize => "Minimize",
            Self::ShowMinNoActive => "ShowMinNoActive",
            Self::ShowNA => "ShowNA",
            Self::Restore => "Restore",
            Self::ShowDefault => "ShowDefault",
            Self::ForceMinimize => "ForceMinimize",
        }
    }
}

impl fmt::Display for ShowWindow {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl TryFrom<i32> for ShowWindow {
    type Error = ValidityError<i32, Self>;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        try_transmute!(value)
    }
}
