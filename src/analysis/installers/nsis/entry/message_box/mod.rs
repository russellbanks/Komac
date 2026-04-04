#![expect(unused)]

mod buttons;
mod icon;

use std::{fmt, fmt::Debug};

pub use buttons::Buttons;
pub use icon::Icon;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LE};

/// * <https://github.com/NSIS-Dev/nsis/blob/v311/Source/Platform.h#L563>
/// * <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-messagebox>
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, Immutable, KnownLayout,
)]
#[repr(transparent)]
pub struct MessageBoxFlags(I32<LE>);

impl MessageBoxFlags {
    const BUTTONS_MASK: i32 = 0xF;
    const ICON_MASK: i32 = 0xF0;
    const DEFAULT_BUTTON_MASK: i32 = 0xF00;

    /// Creates a new [`MessageBoxFlags`].
    #[inline]
    const fn new(mb_flags: i32) -> Self {
        Self(I32::new(mb_flags))
    }

    /// Returns the inner flags.
    #[inline]
    const fn flags(self) -> i32 {
        self.0.get()
    }

    /// Returns the buttons flags.
    #[inline]
    pub(super) const fn buttons_flags(self) -> i32 {
        self.flags() & Self::BUTTONS_MASK
    }

    /// Returns the icon flags.
    #[inline]
    pub(super) const fn icon_flags(self) -> i32 {
        self.flags() & Self::ICON_MASK
    }

    /// Returns the default button flags.
    #[inline]
    pub(super) const fn default_button_flags(self) -> i32 {
        self.flags() & Self::DEFAULT_BUTTON_MASK
    }

    /// Returns the button flags as a [`Buttons`].
    #[inline]
    pub const fn buttons(self) -> Buttons {
        Buttons::new(self)
    }

    /// Returns the icon flags as a [`Icon`].
    #[inline]
    pub const fn icon(self) -> Option<Icon> {
        Icon::new(self)
    }
}

impl fmt::Display for MessageBoxFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.buttons())?;
        if let Some(icon) = self.icon() {
            write!(f, " | {icon}")?;
        }
        Ok(())
    }
}

impl From<I32<LE>> for MessageBoxFlags {
    #[inline]
    fn from(flags: I32<LE>) -> Self {
        Self(flags)
    }
}

impl From<i32> for MessageBoxFlags {
    #[inline]
    fn from(flags: i32) -> Self {
        Self::new(flags)
    }
}
