use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RelativeLocation {
    Root,
    Current,
}

impl RelativeLocation {
    /// Returns `true` if the relative location is [`Root`].
    ///
    /// [`Root`]: Self::Root
    #[must_use]
    #[inline]
    pub const fn is_root(self) -> bool {
        matches!(self, Self::Root)
    }

    /// Returns `true` if the relative location is [`Current`].
    ///
    /// [`Current`]: Self::Current
    #[must_use]
    #[inline]
    pub const fn is_current(self) -> bool {
        matches!(self, Self::Current)
    }
}

impl fmt::Display for RelativeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
