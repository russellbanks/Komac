use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    ops::Range,
};

use itertools::Either;

use super::{
    super::{AnsiStr, WideStr},
    NsisChar, NsisString,
};

/// A borrowed NSIS string in its native encoding: Windows-1252 for ANSI installers and UTF-16LE
/// for Unicode installers.
///
/// Lengths and indices are measured in code units (NSIS `TCHAR`s), matching how NSIS itself
/// measures strings.
#[derive(Copy, Clone)]
pub enum NsisStr<'a> {
    /// A Windows-1252 string from an ANSI installer.
    Ansi(&'a AnsiStr),
    /// A UTF-16LE string from a Unicode installer.
    Wide(&'a WideStr),
}

impl<'a> NsisStr<'a> {
    /// Returns an empty ANSI string.
    #[must_use]
    #[inline]
    pub fn empty() -> Self {
        Self::Ansi(<&AnsiStr>::default())
    }

    /// Returns `$ERROR`, the string NSIS uses when a string cannot be resolved.
    #[must_use]
    #[inline]
    pub fn error() -> Self {
        Self::Ansi(AnsiStr::error())
    }

    /// Creates an `NsisStr` from raw bytes without copying.
    ///
    /// If `unicode` is `true`, the bytes are interpreted as UTF-16LE and any trailing odd byte is
    /// ignored. Otherwise, they are interpreted as Windows-1252.
    #[must_use]
    pub fn from_bytes(bytes: &'a [u8], unicode: bool) -> Self {
        if unicode {
            let even_len = bytes.len() & !1;
            Self::Wide(WideStr::try_from_bytes(&bytes[..even_len]).unwrap_or_default())
        } else {
            Self::Ansi(AnsiStr::from_bytes(bytes))
        }
    }

    /// Returns `true` if this is a UTF-16 string.
    #[must_use]
    #[inline]
    pub const fn is_unicode(self) -> bool {
        matches!(self, Self::Wide(_))
    }

    /// Returns the length of the string in code units.
    #[must_use]
    #[inline]
    pub const fn len(self) -> usize {
        match self {
            Self::Ansi(string) => string.len(),
            Self::Wide(string) => string.len(),
        }
    }

    /// Returns `true` if the string has a length of zero.
    #[must_use]
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns the raw bytes of the string in its native encoding.
    #[must_use]
    #[inline]
    pub fn as_bytes(self) -> &'a [u8] {
        match self {
            Self::Ansi(string) => string.as_bytes(),
            Self::Wide(string) => string.as_bytes(),
        }
    }

    /// Returns the code unit at `index`, or `None` if it is out of bounds.
    #[must_use]
    pub fn char_at(self, index: usize) -> Option<NsisChar> {
        match self {
            Self::Ansi(string) => string.get(index).map(NsisChar::Ansi),
            Self::Wide(string) => string.get(index).map(NsisChar::Wide),
        }
    }

    /// Returns an iterator over the code units of the string.
    pub fn code_units(self) -> impl DoubleEndedIterator<Item = NsisChar> + ExactSizeIterator + 'a {
        match self {
            Self::Ansi(string) => Either::Left(string.iter().copied().map(NsisChar::Ansi)),
            Self::Wide(string) => Either::Right(string.iter().copied().map(NsisChar::Wide)),
        }
    }

    /// Returns the substring in the given range of code units, or `None` if it is out of bounds.
    #[must_use]
    pub fn get(self, range: Range<usize>) -> Option<Self> {
        match self {
            Self::Ansi(string) => string.get_range(range).map(Self::Ansi),
            Self::Wide(string) => string.get_range(range).map(Self::Wide),
        }
    }

    /// Returns the substring in the given range of code units.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds.
    #[must_use]
    pub fn slice(self, range: Range<usize>) -> Self {
        match self {
            Self::Ansi(string) => Self::Ansi(&string[range]),
            Self::Wide(string) => Self::Wide(&string[range]),
        }
    }

    /// Returns an iterator over the characters of the string, decoded from its native encoding.
    pub fn chars(self) -> impl Iterator<Item = char> + 'a {
        match self {
            Self::Ansi(string) => Either::Left(string.chars()),
            Self::Wide(string) => Either::Right(string.chars()),
        }
    }

    /// Decodes the string into UTF-8.
    ///
    /// This borrows the original bytes if the string is ASCII-only ANSI.
    #[must_use]
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        match self {
            Self::Ansi(string) => string.to_string_lossy(),
            Self::Wide(string) => Cow::Owned(string.to_string_lossy()),
        }
    }

    /// Copies the string into an owned [`NsisString`] with the same encoding.
    #[must_use]
    pub fn to_nsis_string(self) -> NsisString {
        match self {
            Self::Ansi(string) => NsisString::Ansi(string.to_owned()),
            Self::Wide(string) => NsisString::Wide(string.to_owned()),
        }
    }

    /// Checks that two strings are an ASCII case-insensitive match, regardless of encoding.
    #[must_use]
    pub fn eq_ignore_ascii_case(self, other: NsisStr<'_>) -> bool {
        match (self, other) {
            (Self::Ansi(a), NsisStr::Ansi(b)) => a.eq_ignore_ascii_case(b),
            (Self::Wide(a), NsisStr::Wide(b)) => a.eq_ignore_ascii_case(b),
            _ => self
                .chars()
                .map(|char| char.to_ascii_lowercase())
                .eq(other.chars().map(|char| char.to_ascii_lowercase())),
        }
    }

    /// Returns `true` if the string starts with `prefix`.
    #[must_use]
    pub fn starts_with(self, prefix: &str) -> bool {
        let mut chars = self.chars();
        prefix
            .chars()
            .all(|expected| chars.next().is_some_and(|char| char == expected))
    }

    /// Returns `true` if the string ends with `suffix`.
    #[must_use]
    pub fn ends_with(self, suffix: &str) -> bool {
        let suffix_len = match self {
            Self::Ansi(_) => suffix.chars().count(),
            Self::Wide(_) => suffix.encode_utf16().count(),
        };
        self.len()
            .checked_sub(suffix_len)
            .is_some_and(|start| self.slice(start..self.len()) == *suffix)
    }

    /// Returns `true` if the string starts with the ASCII `prefix`, ignoring ASCII case.
    #[must_use]
    pub fn starts_with_ignore_ascii_case(self, prefix: &str) -> bool {
        match self {
            Self::Ansi(string) => string.starts_with_ignore_ascii_case(prefix),
            Self::Wide(string) => string.starts_with_ignore_ascii_case(prefix),
        }
    }

    /// Parses the string as an integer in the given radix, as [`i32::from_str_radix`] would.
    #[must_use]
    pub fn parse_int(self, radix: u32) -> Option<i32> {
        match self {
            Self::Ansi(string) => string.parse_int(radix),
            Self::Wide(string) => string.parse_int(radix),
        }
    }
}

impl Default for NsisStr<'_> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a> From<&'a AnsiStr> for NsisStr<'a> {
    #[inline]
    fn from(value: &'a AnsiStr) -> Self {
        Self::Ansi(value)
    }
}

impl<'a> From<&'a WideStr> for NsisStr<'a> {
    #[inline]
    fn from(value: &'a WideStr) -> Self {
        Self::Wide(value)
    }
}

impl PartialEq<NsisStr<'_>> for NsisStr<'_> {
    fn eq(&self, other: &NsisStr<'_>) -> bool {
        match (*self, *other) {
            (Self::Ansi(a), NsisStr::Ansi(b)) => a == b,
            (Self::Wide(a), NsisStr::Wide(b)) => a == b,
            (a, b) => a.chars().eq(b.chars()),
        }
    }
}

impl Eq for NsisStr<'_> {}

impl PartialEq<str> for NsisStr<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        match *self {
            Self::Ansi(string) => *string == *other,
            Self::Wide(string) => *string == *other,
        }
    }
}

impl PartialEq<&str> for NsisStr<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self == **other
    }
}

impl PartialEq<NsisStr<'_>> for str {
    #[inline]
    fn eq(&self, other: &NsisStr<'_>) -> bool {
        *other == *self
    }
}

impl PartialEq<NsisStr<'_>> for &str {
    #[inline]
    fn eq(&self, other: &NsisStr<'_>) -> bool {
        *other == **self
    }
}

impl Hash for NsisStr<'_> {
    /// Hashes the decoded characters so that equal ANSI and Unicode strings hash identically.
    fn hash<H: Hasher>(&self, state: &mut H) {
        for char in self.chars() {
            char.hash(state);
        }
        state.write_u8(0xFF);
    }
}

impl fmt::Display for NsisStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ansi(string) => fmt::Display::fmt(string, f),
            Self::Wide(string) => fmt::Display::fmt(string, f),
        }
    }
}

impl fmt::Debug for NsisStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ansi(string) => fmt::Debug::fmt(string, f),
            Self::Wide(string) => fmt::Debug::fmt(string, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, RandomState};

    use super::NsisStr;

    const UNICODE: &[u8] = b"a\0 \0s\0t\0r\0i\0n\0g\0";

    #[test]
    fn from_bytes() {
        let ansi = NsisStr::from_bytes(b"a string", false);
        let wide = NsisStr::from_bytes(UNICODE, true);
        assert!(!ansi.is_unicode());
        assert!(wide.is_unicode());
        assert_eq!(ansi.len(), 8);
        assert_eq!(wide.len(), 8);
        assert_eq!(ansi, "a string");
        assert_eq!(wide, "a string");

        // A trailing odd byte is ignored
        assert_eq!(NsisStr::from_bytes(b"a\0b", true), "a");
    }

    #[test]
    fn cross_encoding_equality() {
        let ansi = NsisStr::from_bytes(b"\x80", false);
        let euro = 0x20ACu16.to_le_bytes();
        let wide = NsisStr::from_bytes(&euro, true);
        assert_eq!(ansi, wide);
        assert_ne!(ansi, NsisStr::from_bytes(b"x\0", true));

        let hasher = RandomState::new();
        assert_eq!(hasher.hash_one(ansi), hasher.hash_one(wide));
    }

    #[test]
    fn slicing() {
        // Mirrors `StrCpy $0 "a string" 3 -4 # = "rin"`
        for string in [
            NsisStr::from_bytes(b"a string", false),
            NsisStr::from_bytes(UNICODE, true),
        ] {
            assert_eq!(string.slice(4..7), "rin");
            assert_eq!(string.get(4..9), None);
            assert_eq!(string.char_at(0).map(|char| char.to_ansi()), Some(b'a'));
        }
    }

    #[test]
    fn ascii_case() {
        let ansi = NsisStr::from_bytes(b"HELLO", false);
        let wide = NsisStr::from_bytes(b"h\0e\0l\0l\0o\0", true);
        assert!(ansi.eq_ignore_ascii_case(wide));
        assert!(wide.eq_ignore_ascii_case(wide));
        assert!(wide.starts_with_ignore_ascii_case("HE"));
        assert!(wide.starts_with("he"));
        assert!(!wide.starts_with("HE"));
    }

    #[test]
    fn parse_int() {
        assert_eq!(NsisStr::from_bytes(b"42", false).parse_int(10), Some(42));
        assert_eq!(
            NsisStr::from_bytes(b"4\x002\0", true).parse_int(10),
            Some(42)
        );
        assert_eq!(NsisStr::from_bytes(b"x", false).parse_int(10), None);
    }
}
