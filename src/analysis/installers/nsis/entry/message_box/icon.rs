use std::{fmt, ops::BitOr};

use super::{Buttons, MessageBoxFlags};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Icon {
    Stop = 0x10,
    Question = 0x20,
    Exclamation = 0x30,
    Information = 0x40,
    User = 0x80,
}

impl Icon {
    pub const fn new(mb_flags: MessageBoxFlags) -> Option<Self> {
        match mb_flags.icon_flags() {
            0x10 => Some(Self::Stop),
            0x20 => Some(Self::Question),
            0x30 => Some(Self::Exclamation),
            0x40 => Some(Self::Information),
            0x80 => Some(Self::User),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stop => "ICONSTOP",
            Self::Question => "ICONQUESTION",
            Self::Exclamation => "ICONEXCLAMATION",
            Self::Information => "ICONINFORMATION",
            Self::User => "USERICON",
        }
    }
}

impl BitOr<Buttons> for Icon {
    type Output = MessageBoxFlags;

    fn bitor(self, rhs: Buttons) -> Self::Output {
        MessageBoxFlags::from(self as i32 | rhs as i32)
    }
}

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<Icon> for MessageBoxFlags {
    #[inline]
    fn from(icon: Icon) -> Self {
        Self::from(icon as i32)
    }
}
