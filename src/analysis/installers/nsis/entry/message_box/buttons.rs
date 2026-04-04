use std::{fmt, ops::BitOr};

use super::{Icon, MessageBoxFlags};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Buttons {
    Ok = 0,
    OkCancel = 1,
    AbortRetryIgnore = 2,
    YesNoCancel = 3,
    YesNo = 4,
    RetryCancel = 5,
    CancelTryContinue = 6,
}

impl Buttons {
    /// Creates a new [`Buttons`] flag.
    pub const fn new(mb_flags: MessageBoxFlags) -> Self {
        match mb_flags.buttons_flags() {
            1 => Self::OkCancel,
            2 => Self::AbortRetryIgnore,
            3 => Self::YesNoCancel,
            4 => Self::YesNo,
            5 => Self::RetryCancel,
            6 => Self::CancelTryContinue,
            _ => Self::Ok,
        }
    }

    /// Returns the Message Box buttons as a static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::OkCancel => "OKCANCEL",
            Self::AbortRetryIgnore => "ABORTRETRYIGNORE",
            Self::YesNoCancel => "YESNOCANCEL",
            Self::YesNo => "YESNO",
            Self::RetryCancel => "RETRYCANCEL",
            Self::CancelTryContinue => "CANCELTRYCONTINUE",
        }
    }
}

impl BitOr<Icon> for Buttons {
    type Output = MessageBoxFlags;

    fn bitor(self, icon: Icon) -> Self::Output {
        MessageBoxFlags::from(self as i32 | icon as i32)
    }
}

impl fmt::Display for Buttons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<MessageBoxFlags> for Buttons {
    #[inline]
    fn from(mb_flags: MessageBoxFlags) -> Self {
        Self::new(mb_flags)
    }
}

impl From<Buttons> for MessageBoxFlags {
    #[inline]
    fn from(buttons: Buttons) -> Self {
        Self::from(buttons as i32)
    }
}
