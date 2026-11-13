use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
};

use super::{NsisStr, NsisString};

/// A clone-on-write NSIS string, which is either borrowed from the installer's string block or
/// owned after being built up from variables and other strings.
///
/// This exists because [`NsisStr`] is an enum of references rather than an unsized type, so it
/// cannot be used with [`std::borrow::Cow`].
#[derive(Clone)]
pub enum NsisCow<'a> {
    /// A string borrowed from the installer's data.
    Borrowed(NsisStr<'a>),
    /// An owned string.
    Owned(NsisString),
}

impl<'a> NsisCow<'a> {
    /// Returns a borrowed view of the string.
    #[must_use]
    #[inline]
    pub fn as_nsis_str(&self) -> NsisStr<'_> {
        match self {
            Self::Borrowed(string) => *string,
            Self::Owned(string) => string.as_nsis_str(),
        }
    }

    /// Returns `true` if the string is borrowed.
    #[must_use]
    #[inline]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns `true` if the string is owned.
    #[must_use]
    #[inline]
    pub const fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Returns `true` if this is a UTF-16 string.
    #[must_use]
    #[inline]
    pub fn is_unicode(&self) -> bool {
        self.as_nsis_str().is_unicode()
    }

    /// Returns the length of the string in code units.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.as_nsis_str().len()
    }

    /// Returns `true` if the string has a length of zero.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_nsis_str().is_empty()
    }

    /// Extracts the owned string, cloning the borrowed data if necessary.
    #[must_use]
    pub fn into_owned(self) -> NsisString {
        match self {
            Self::Borrowed(string) => string.to_nsis_string(),
            Self::Owned(string) => string,
        }
    }

    /// Returns a mutable reference to the owned string, cloning the borrowed data if necessary.
    pub fn to_mut(&mut self) -> &mut NsisString {
        if let Self::Borrowed(string) = *self {
            *self = Self::Owned(string.to_nsis_string());
        }

        match self {
            Self::Owned(string) => string,
            Self::Borrowed(_) => unreachable!(),
        }
    }

    /// Decodes the string into UTF-8.
    #[must_use]
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_nsis_str().to_string_lossy()
    }

    /// Decodes the string into an owned UTF-8 [`String`].
    #[must_use]
    pub fn into_string_lossy(self) -> String {
        match self {
            Self::Borrowed(string) => string.to_string_lossy().into_owned(),
            Self::Owned(string) => string.into_string_lossy(),
        }
    }
}

impl Default for NsisCow<'_> {
    #[inline]
    fn default() -> Self {
        Self::Borrowed(NsisStr::empty())
    }
}

impl<'a> From<NsisStr<'a>> for NsisCow<'a> {
    #[inline]
    fn from(value: NsisStr<'a>) -> Self {
        Self::Borrowed(value)
    }
}

impl From<NsisString> for NsisCow<'_> {
    #[inline]
    fn from(value: NsisString) -> Self {
        Self::Owned(value)
    }
}

impl From<NsisCow<'_>> for NsisString {
    #[inline]
    fn from(value: NsisCow<'_>) -> Self {
        value.into_owned()
    }
}

impl PartialEq for NsisCow<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_nsis_str() == other.as_nsis_str()
    }
}

impl Eq for NsisCow<'_> {}

impl PartialEq<NsisStr<'_>> for NsisCow<'_> {
    #[inline]
    fn eq(&self, other: &NsisStr<'_>) -> bool {
        self.as_nsis_str() == *other
    }
}

impl PartialEq<NsisString> for NsisCow<'_> {
    #[inline]
    fn eq(&self, other: &NsisString) -> bool {
        self.as_nsis_str() == other.as_nsis_str()
    }
}

impl PartialEq<str> for NsisCow<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_nsis_str() == *other
    }
}

impl PartialEq<&str> for NsisCow<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_nsis_str() == **other
    }
}

impl Hash for NsisCow<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_nsis_str().hash(state);
    }
}

impl fmt::Display for NsisCow<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_nsis_str(), f)
    }
}

impl fmt::Debug for NsisCow<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_nsis_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::{NsisCow, NsisStr};

    #[test]
    fn to_mut() {
        let mut cow = NsisCow::from(NsisStr::from_bytes(b"abc", false));
        assert!(cow.is_borrowed());
        cow.to_mut().push_str("def");
        assert!(cow.is_owned());
        assert_eq!(cow, "abcdef");
        assert_eq!(cow.into_string_lossy(), "abcdef");
    }
}
