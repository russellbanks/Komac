use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes, ValidityError, try_transmute};

/// A merge of [`Window Messages`] and [`Window Notifications`].
///
/// [`Window Messages`]: https://learn.microsoft.com/windows/win32/winmsg/window-messages
/// [`Window Notifications`]: https://learn.microsoft.com/windows/win32/winmsg/window-notifications
#[expect(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u16)]
pub enum WindowMessage {
    Null = 0x0000,
    Create = 0x0001,
    Destroy = 0x0002,
    Move = 0x0003,
    Size = 0x0005,
    Enable = 0x000A,
    SetText = 0x000C,
    GetText = 0x000D,
    GetTextLength = 0x000E,
    Close = 0x0010,
    Quit = 0x0012,
    QueryOpen = 0x0013,
    ShowWindow = 0x0018,
    ActivateApp = 0x001C,
    CancelMode = 0x001F,
    ChildActivate = 0x0022,
    GetMinMaxInfo = 0x0024,
    SetFont = 0x0030,
    GetFont = 0x0031,
    QueryDragIcon = 0x0037,
    Compacting = 0x0041,
    WindowPosChanging = 0x0046,
    WindowPosChanged = 0x0047,
    InputLangChangeRequest = 0x0050,
    InputLangChange = 0x0051,
    UserChanged = 0x0054,
    StyleChanging = 0x007C,
    StyleChanged = 0x007D,
    GetIcon = 0x007F,
    SetIcon = 0x0080,
    NCCreate = 0x0081,
    NCDestroy = 0x0082,
    NCCalcSize = 0x0083,
    NCActivate = 0x0086,
    GetHMenu = 0x01E1,
    Sizing = 0x0214,
    Moving = 0x0216,
    EnterSizeMove = 0x0231,
    ExitSizeMove = 0x0232,
    ThemeChanged = 0x031A,
}

impl WindowMessage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Create => "Create",
            Self::Destroy => "Destroy",
            Self::Move => "Move",
            Self::Size => "Size",
            Self::Enable => "Enable",
            Self::SetText => "SetText",
            Self::GetText => "GetText",
            Self::GetTextLength => "GetTextLength",
            Self::Close => "Close",
            Self::Quit => "Quit",
            Self::QueryOpen => "QueryOpen",
            Self::ShowWindow => "ShowWindow",
            Self::ActivateApp => "ActivateApp",
            Self::CancelMode => "CancelMode",
            Self::ChildActivate => "ChildActivate",
            Self::GetMinMaxInfo => "GetMinMaxInfo",
            Self::SetFont => "SetFont",
            Self::GetFont => "GetFont",
            Self::QueryDragIcon => "QueryDragIcon",
            Self::Compacting => "Compacting",
            Self::WindowPosChanging => "WindowPosChanging",
            Self::WindowPosChanged => "WindowPosChanged",
            Self::InputLangChangeRequest => "InputLangChangeRequest",
            Self::InputLangChange => "InputLangChange",
            Self::UserChanged => "UserChanged",
            Self::StyleChanging => "StyleChanging",
            Self::StyleChanged => "StyleChanged",
            Self::GetIcon => "GetIcon",
            Self::SetIcon => "SetIcon",
            Self::NCCreate => "NCCreate",
            Self::NCDestroy => "NCDestroy",
            Self::NCCalcSize => "NCCalcSize",
            Self::NCActivate => "NCActivate",
            Self::GetHMenu => "GetHMenu",
            Self::Sizing => "Sizing",
            Self::Moving => "Moving",
            Self::EnterSizeMove => "EnterSizeMove",
            Self::ExitSizeMove => "ExitSizeMove",
            Self::ThemeChanged => "ThemeChanged",
        }
    }
}

impl fmt::Debug for WindowMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}({})", *self as u16)
    }
}

impl fmt::Display for WindowMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl TryFrom<u16> for WindowMessage {
    type Error = ValidityError<u16, Self>;

    #[inline]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        try_transmute!(value)
    }
}
