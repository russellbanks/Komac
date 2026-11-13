use std::{
    fmt,
    hash::{Hash, Hasher},
};

use super::{
    super::{AnsiString, WideString, encode_char},
    NsisChar, NsisStr,
};

/// An owned NSIS string in its native encoding: Windows-1252 for ANSI installers and UTF-16LE
/// for Unicode installers.
#[derive(Clone)]
pub enum NsisString {
    /// A Windows-1252 string from an ANSI installer.
    Ansi(AnsiString),
    /// A UTF-16LE string from a Unicode installer.
    Wide(WideString),
}

impl NsisString {
    /// Creates a new empty ANSI string. This does not allocate.
    #[must_use]
    #[inline]
    pub const fn new_ansi() -> Self {
        Self::Ansi(AnsiString::new())
    }

    /// Creates a new empty Unicode string. This does not allocate.
    #[must_use]
    #[inline]
    pub const fn new_wide() -> Self {
        Self::Wide(WideString::new())
    }

    /// Creates a new empty string, which is Unicode if `unicode` is `true` and ANSI otherwise.
    #[must_use]
    #[inline]
    pub const fn new(unicode: bool) -> Self {
        if unicode {
            Self::new_wide()
        } else {
            Self::new_ansi()
        }
    }

    /// Encodes a UTF-8 string as Unicode if `unicode` is `true`, or as Windows-1252 otherwise.
    #[must_use]
    pub fn from_str(string: &str, unicode: bool) -> Self {
        if unicode {
            Self::Wide(WideString::from(string))
        } else {
            Self::Ansi(AnsiString::from(string))
        }
    }

    /// Returns a borrowed [`NsisStr`] view of the entire string.
    #[must_use]
    #[inline]
    pub fn as_nsis_str(&self) -> NsisStr<'_> {
        match self {
            Self::Ansi(string) => NsisStr::Ansi(string),
            Self::Wide(string) => NsisStr::Wide(string),
        }
    }

    /// Returns `true` if this is a UTF-16 string.
    #[must_use]
    #[inline]
    pub const fn is_unicode(&self) -> bool {
        matches!(self, Self::Wide(_))
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
        self.len() == 0
    }

    /// Appends a single code unit, converting it to this string's encoding if necessary.
    pub fn push(&mut self, value: NsisChar) {
        match self {
            Self::Ansi(dst) => dst.push(value.to_ansi()),
            Self::Wide(dst) => dst.push(value.to_wide()),
        }
    }

    /// Appends a [`char`], encoding it in this string's encoding.
    pub fn push_char(&mut self, value: char) {
        match self {
            Self::Ansi(dst) => dst.push_char(value),
            Self::Wide(dst) => dst.push_char(value),
        }
    }

    /// Appends a string, converting it to this string's encoding if necessary.
    pub fn push_nsis_str(&mut self, value: NsisStr<'_>) {
        match (self, value) {
            (Self::Ansi(dst), NsisStr::Ansi(src)) => dst.push_ansi_str(src),
            (Self::Wide(dst), NsisStr::Wide(src)) => dst.push_str(src),
            (Self::Ansi(dst), NsisStr::Wide(src)) => dst.extend(src.chars().map(encode_char)),
            (Self::Wide(dst), NsisStr::Ansi(src)) => dst.push_str(&src.to_wide()),
        }
    }

    /// Appends a UTF-8 string, encoding it in this string's encoding.
    pub fn push_str(&mut self, value: &str) {
        match self {
            Self::Ansi(dst) => dst.push_str(value),
            Self::Wide(dst) => dst.push_str_utf8(value),
        }
    }

    /// Shortens the string to `new_len` code units. Does nothing if `new_len` is greater than the
    /// string's current length.
    pub fn truncate(&mut self, new_len: usize) {
        match self {
            Self::Ansi(string) => string.truncate(new_len),
            Self::Wide(string) => string.truncate(new_len),
        }
    }

    /// Removes all code units from the string, keeping its allocated capacity.
    pub fn clear(&mut self) {
        match self {
            Self::Ansi(string) => string.clear(),
            Self::Wide(string) => string.clear(),
        }
    }

    /// Decodes the string into an owned UTF-8 [`String`].
    #[must_use]
    pub fn into_string_lossy(self) -> String {
        match self {
            Self::Ansi(string) => string.into_string_lossy(),
            Self::Wide(string) => string.into_string_lossy(),
        }
    }
}

impl Default for NsisString {
    #[inline]
    fn default() -> Self {
        Self::new_ansi()
    }
}

impl From<AnsiString> for NsisString {
    #[inline]
    fn from(value: AnsiString) -> Self {
        Self::Ansi(value)
    }
}

impl From<WideString> for NsisString {
    #[inline]
    fn from(value: WideString) -> Self {
        Self::Wide(value)
    }
}

impl From<NsisStr<'_>> for NsisString {
    #[inline]
    fn from(value: NsisStr<'_>) -> Self {
        value.to_nsis_string()
    }
}

impl PartialEq for NsisString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_nsis_str() == other.as_nsis_str()
    }
}

impl Eq for NsisString {}

impl PartialEq<NsisStr<'_>> for NsisString {
    #[inline]
    fn eq(&self, other: &NsisStr<'_>) -> bool {
        self.as_nsis_str() == *other
    }
}

impl PartialEq<str> for NsisString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_nsis_str() == *other
    }
}

impl PartialEq<&str> for NsisString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_nsis_str() == **other
    }
}

impl Hash for NsisString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_nsis_str().hash(state);
    }
}

impl fmt::Display for NsisString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_nsis_str(), f)
    }
}

impl fmt::Debug for NsisString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_nsis_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::{NsisChar, NsisStr, NsisString};

    #[test]
    fn push_across_encodings() {
        let mut ansi = NsisString::new_ansi();
        ansi.push_nsis_str(NsisStr::from_bytes(b"a\0\xAC\x20\x01\x01", true)); // "a€ā"
        ansi.push_str("€");
        ansi.push(NsisChar::Ansi(b'!'));
        assert_eq!(ansi.as_nsis_str().as_bytes(), b"a\x80?\x80!");

        let mut wide = NsisString::new_wide();
        wide.push_nsis_str(NsisStr::from_bytes(b"a\x80", false));
        wide.push_char('😀');
        wide.push(NsisChar::Ansi(0x80));
        assert_eq!(wide, "a€😀€");
        assert_eq!(wide.len(), 5);
        assert_eq!(wide.into_string_lossy(), "a€😀€");
    }

    #[test]
    fn from_str() {
        assert!(NsisString::from_str("x", true).is_unicode());
        assert!(!NsisString::from_str("x", false).is_unicode());
        assert_eq!(
            NsisString::from_str("€", false),
            NsisString::from_str("€", true)
        );
    }
}
