use std::{
    borrow::Cow,
    fmt,
    ops::{Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    slice,
};

use encoding_rs::WINDOWS_1252;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, transmute_ref};

use super::{
    super::{WideChar, WideString},
    AnsiChar, AnsiString,
};

/// A borrowed ANSI string, stored as raw Windows-1252 code units.
///
/// This is the unsized counterpart to [`AnsiString`], in the same way that [`str`] is to
/// [`String`]. Lengths and indices are measured in bytes (ANSI code units).
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AnsiStr([AnsiChar]);

impl AnsiStr {
    /// The string NSIS uses when a string cannot be resolved.
    #[must_use]
    #[inline]
    pub fn error() -> &'static Self {
        Self::from_bytes(b"$ERROR")
    }

    /// Creates an `AnsiStr` from raw Windows-1252 bytes without copying.
    #[must_use]
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        transmute_ref!(bytes)
    }

    /// Returns the length of the string in bytes.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the string has a length of zero.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the raw Windows-1252 bytes of the string.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Decodes the string from Windows-1252 into UTF-8.
    ///
    /// This borrows the original bytes if the string is entirely ASCII.
    #[must_use]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        WINDOWS_1252.decode_without_bom_handling(&self.0).0
    }

    /// Converts the string into a UTF-16 [`WideString`], decoding each byte as Windows-1252.
    #[must_use]
    pub fn to_wide(&self) -> WideString {
        self.to_string_lossy()
            .encode_utf16()
            .map(WideChar::new)
            .collect()
    }

    /// Copies the string into an owned [`AnsiString`].
    #[must_use]
    #[inline]
    pub fn to_ansi_string(&self) -> AnsiString {
        self.to_owned()
    }

    /// Returns an iterator over the characters of the string, decoded as Windows-1252.
    pub fn chars(&self) -> impl DoubleEndedIterator<Item = char> + ExactSizeIterator + '_ {
        self.0.iter().map(|&byte| decode_char(byte))
    }

    /// Returns an iterator over the raw bytes of the string.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, AnsiChar> {
        self.0.iter()
    }

    /// Returns the first byte of the string, or `None` if it is empty.
    #[must_use]
    #[inline]
    pub fn first(&self) -> Option<AnsiChar> {
        self.0.first().copied()
    }

    /// Returns the last byte of the string, or `None` if it is empty.
    #[must_use]
    #[inline]
    pub fn last(&self) -> Option<AnsiChar> {
        self.0.last().copied()
    }

    /// Returns the byte at `index`, or `None` if it is out of bounds.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<AnsiChar> {
        self.0.get(index).copied()
    }

    /// Returns the substring in `range`, or `None` if it is out of bounds.
    #[must_use]
    #[inline]
    pub fn get_range(&self, range: Range<usize>) -> Option<&Self> {
        self.0.get(range).map(Self::from_bytes)
    }

    /// Returns `true` if the string starts with `prefix`, comparing raw bytes.
    #[must_use]
    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.0.starts_with(&prefix.0)
    }

    /// Returns `true` if the string ends with `suffix`, comparing raw bytes.
    #[must_use]
    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        self.0.ends_with(&suffix.0)
    }

    /// Returns `true` if the string contains the byte `needle`.
    #[must_use]
    #[inline]
    pub fn contains(&self, needle: AnsiChar) -> bool {
        self.0.contains(&needle)
    }

    /// Checks that two strings are an ASCII case-insensitive match.
    #[must_use]
    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Returns `true` if the string starts with the ASCII `prefix`, ignoring ASCII case.
    #[must_use]
    pub fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool {
        self.0
            .get(..prefix.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(prefix.as_bytes()))
    }

    /// Parses the string as an integer in the given radix, as [`i32::from_str_radix`] would.
    #[must_use]
    pub fn parse_int(&self, radix: u32) -> Option<i32> {
        std::str::from_utf8(&self.0)
            .ok()
            .and_then(|string| i32::from_str_radix(string, radix).ok())
    }
}

/// Decodes a single Windows-1252 byte into a [`char`].
pub(in super::super) fn decode_char(byte: AnsiChar) -> char {
    if byte.is_ascii() {
        char::from(byte)
    } else {
        let mut buf = [0; 4];
        let (_, _, written, _) = WINDOWS_1252
            .new_decoder_without_bom_handling()
            .decode_to_utf8(&[byte], &mut buf, true);
        std::str::from_utf8(&buf[..written])
            .ok()
            .and_then(|string| string.chars().next())
            .unwrap_or(char::REPLACEMENT_CHARACTER)
    }
}

/// Encodes a single [`char`] into Windows-1252, returning `?` if it is not representable.
pub(in super::super) fn encode_char(char: char) -> AnsiChar {
    if char.is_ascii() {
        return char as AnsiChar;
    }

    let mut utf8 = [0; 4];
    let (bytes, _, had_errors) = WINDOWS_1252.encode(char.encode_utf8(&mut utf8));
    match &*bytes {
        [byte] if !had_errors => *byte,
        _ => b'?',
    }
}

impl ToOwned for AnsiStr {
    type Owned = AnsiString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        AnsiString::from(self.0.to_vec())
    }
}

impl Default for &AnsiStr {
    #[inline]
    fn default() -> Self {
        AnsiStr::from_bytes(&[])
    }
}

impl AsRef<Self> for AnsiStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for AnsiStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&'a [u8]> for &'a AnsiStr {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        AnsiStr::from_bytes(value)
    }
}

impl PartialEq<str> for AnsiStr {
    fn eq(&self, other: &str) -> bool {
        if other.is_ascii() {
            self.0 == *other.as_bytes()
        } else {
            self.chars().eq(other.chars())
        }
    }
}

impl PartialEq<&str> for AnsiStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self == **other
    }
}

impl PartialEq<AnsiStr> for str {
    #[inline]
    fn eq(&self, other: &AnsiStr) -> bool {
        *other == *self
    }
}

impl Index<usize> for AnsiStr {
    type Output = AnsiChar;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

macro_rules! impl_ansi_str_index {
    ($($range:ty),* $(,)?) => {
        $(
            impl Index<$range> for AnsiStr {
                type Output = Self;

                #[inline]
                fn index(&self, index: $range) -> &Self::Output {
                    Self::from_bytes(&self.0[index])
                }
            }
        )*
    };
}

impl_ansi_str_index!(
    Range<usize>,
    RangeFrom<usize>,
    RangeTo<usize>,
    RangeFull,
    RangeInclusive<usize>,
    RangeToInclusive<usize>,
);

impl fmt::Display for AnsiStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_string_lossy().fmt(f)
    }
}

impl fmt::Debug for AnsiStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.to_string_lossy(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::AnsiStr;

    #[test]
    fn windows_1252_decoding() {
        let string = AnsiStr::from_bytes(b"\x80 \x9F \xE9");
        assert_eq!(string.to_string_lossy(), "€ Ÿ é");
        assert_eq!(string.to_string(), "€ Ÿ é");
        assert_eq!(string.chars().collect::<String>(), "€ Ÿ é");
        assert_eq!(*string, *"€ Ÿ é");
    }

    #[test]
    fn to_wide() {
        let wide = AnsiStr::from_bytes(b"a\x80").to_wide();
        assert_eq!(wide.to_string_lossy(), "a€");
        assert_eq!(wide.len(), 2);
    }

    #[test]
    fn indexing() {
        let string = AnsiStr::from_bytes(b"a string");
        assert_eq!(&string[2..], "string");
        assert_eq!(&string[..1], "a");
        assert_eq!(string.get_range(2..20), None);
        assert_eq!(string.get(0), Some(b'a'));
    }

    #[test]
    fn ascii_case() {
        let string = AnsiStr::from_bytes(b"0xFF");
        assert!(string.starts_with_ignore_ascii_case("0X"));
        assert!(string.eq_ignore_ascii_case(AnsiStr::from_bytes(b"0Xff")));
        assert!(!AnsiStr::from_bytes(b"0").starts_with_ignore_ascii_case("0x"));
    }

    #[test]
    fn parse_int() {
        assert_eq!(AnsiStr::from_bytes(b"123").parse_int(10), Some(123));
        assert_eq!(AnsiStr::from_bytes(b"-5").parse_int(10), Some(-5));
        assert_eq!(AnsiStr::from_bytes(b"FF").parse_int(16), Some(0xFF));
        assert_eq!(AnsiStr::from_bytes(b"abc").parse_int(10), None);
        assert_eq!(AnsiStr::from_bytes(b"\xFF").parse_int(10), None);
    }
}
