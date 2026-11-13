use std::{
    fmt,
    fmt::Write,
    ops::{Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    slice,
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, transmute_ref};

use super::{
    super::{AnsiString, encode_char},
    WideChar, WideString,
};

/// A borrowed UTF-16LE string.
///
/// This is the unsized counterpart to [`WideString`], in the same way that [`str`] is to
/// [`String`]. Lengths and indices are measured in UTF-16 code units, and the string is not
/// guaranteed to be valid UTF-16.
#[derive(PartialEq, Eq, Hash, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct WideStr([WideChar]);

impl WideStr {
    /// Creates a `WideStr` from a slice of UTF-16LE code units without copying.
    #[must_use]
    #[inline]
    pub fn from_units(units: &[WideChar]) -> &Self {
        transmute_ref!(units)
    }

    /// Creates a `WideStr` from raw UTF-16LE bytes without copying.
    ///
    /// Returns `None` if the length of `bytes` is not a multiple of two.
    #[must_use]
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Option<&Self> {
        Self::ref_from_bytes(bytes).ok()
    }

    /// Returns the length of the string in UTF-16 code units.
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

    /// Returns the UTF-16LE code units of the string.
    #[must_use]
    #[inline]
    pub const fn as_units(&self) -> &[WideChar] {
        &self.0
    }

    /// Returns the raw UTF-16LE bytes of the string.
    #[must_use]
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns an iterator over the UTF-16LE code units of the string.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, WideChar> {
        self.0.iter()
    }

    /// Returns the code unit at `index`, or `None` if it is out of bounds.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<WideChar> {
        self.0.get(index).copied()
    }

    /// Returns the substring in `range`, or `None` if it is out of bounds.
    #[must_use]
    #[inline]
    pub fn get_range(&self, range: Range<usize>) -> Option<&Self> {
        self.0.get(range).map(Self::from_units)
    }

    /// Returns the first code unit of the string, or `None` if it is empty.
    #[must_use]
    #[inline]
    pub fn first(&self) -> Option<WideChar> {
        self.0.first().copied()
    }

    /// Returns the last code unit of the string, or `None` if it is empty.
    #[must_use]
    #[inline]
    pub fn last(&self) -> Option<WideChar> {
        self.0.last().copied()
    }

    /// Returns an iterator over the characters of the string.
    ///
    /// Unpaired surrogates are replaced with [`char::REPLACEMENT_CHARACTER`].
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        char::decode_utf16(self.0.iter().map(|unit| unit.get()))
            .map(|result| result.unwrap_or(char::REPLACEMENT_CHARACTER))
    }

    /// Decodes the string into UTF-8, replacing unpaired surrogates with
    /// [`char::REPLACEMENT_CHARACTER`].
    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        self.chars().collect()
    }

    /// Converts the string into a Windows-1252 [`AnsiString`], replacing characters that are not
    /// representable with `?`.
    #[must_use]
    pub fn to_ansi_lossy(&self) -> AnsiString {
        self.chars().map(encode_char).collect()
    }

    /// Copies the string into an owned [`WideString`].
    #[must_use]
    #[inline]
    pub fn to_wide_string(&self) -> WideString {
        self.to_owned()
    }

    /// Returns `true` if the string starts with `prefix`, comparing code units.
    #[must_use]
    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.0.starts_with(&prefix.0)
    }

    /// Returns `true` if the string ends with `suffix`, comparing code units.
    #[must_use]
    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        self.0.ends_with(&suffix.0)
    }

    /// Returns `true` if the string contains the code unit `value`.
    #[must_use]
    #[inline]
    pub fn contains(&self, value: WideChar) -> bool {
        self.0.contains(&value)
    }

    /// Checks that two strings are an ASCII case-insensitive match.
    #[must_use]
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(&a, &b)| eq_unit_ignore_ascii_case(a.get(), b.get()))
    }

    /// Returns `true` if the string starts with the ASCII `prefix`, ignoring ASCII case.
    #[must_use]
    pub fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool {
        let mut units = self.iter();
        prefix.encode_utf16().all(|expected| {
            units
                .next()
                .is_some_and(|unit| eq_unit_ignore_ascii_case(unit.get(), expected))
        })
    }

    /// Parses the string as an integer in the given radix, as [`i32::from_str_radix`] would.
    #[must_use]
    pub fn parse_int(&self, radix: u32) -> Option<i32> {
        let ascii = self
            .iter()
            .map(|unit| u8::try_from(unit.get()).ok().filter(u8::is_ascii))
            .collect::<Option<Vec<_>>>()?;
        std::str::from_utf8(&ascii)
            .ok()
            .and_then(|string| i32::from_str_radix(string, radix).ok())
    }
}

/// Checks that two UTF-16 code units are equal, ignoring ASCII case.
fn eq_unit_ignore_ascii_case(a: u16, b: u16) -> bool {
    /// Converts an ASCII uppercase code unit to lowercase, leaving every other code unit unchanged.
    const fn to_ascii_lowercase(unit: u16) -> u16 {
        match unit {
            0x41..=0x5A => unit + 0x20, // A-Z => a-z
            _ => unit,
        }
    }

    to_ascii_lowercase(a) == to_ascii_lowercase(b)
}

impl ToOwned for WideStr {
    type Owned = WideString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        WideString::from(self.0.to_vec())
    }
}

impl Default for &WideStr {
    #[inline]
    fn default() -> Self {
        WideStr::from_units(&[])
    }
}

impl AsRef<Self> for WideStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[WideChar]> for WideStr {
    #[inline]
    fn as_ref(&self) -> &[WideChar] {
        &self.0
    }
}

impl AsRef<[u8]> for WideStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&'a [WideChar]> for &'a WideStr {
    #[inline]
    fn from(value: &'a [WideChar]) -> Self {
        WideStr::from_units(value)
    }
}

impl PartialEq<str> for WideStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.iter().map(|unit| unit.get()).eq(other.encode_utf16())
    }
}

impl PartialEq<&str> for WideStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self == **other
    }
}

impl PartialEq<WideStr> for str {
    #[inline]
    fn eq(&self, other: &WideStr) -> bool {
        *other == *self
    }
}

impl Index<usize> for WideStr {
    type Output = WideChar;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

macro_rules! impl_wide_str_index {
    ($($range:ty),* $(,)?) => {
        $(
            impl Index<$range> for WideStr {
                type Output = Self;

                #[inline]
                fn index(&self, index: $range) -> &Self::Output {
                    Self::from_units(&self.0[index])
                }
            }
        )*
    };
}

impl_wide_str_index!(
    Range<usize>,
    RangeFrom<usize>,
    RangeTo<usize>,
    RangeFull,
    RangeInclusive<usize>,
    RangeToInclusive<usize>,
);

impl fmt::Display for WideStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.width().is_some() || f.precision().is_some() {
            return self.to_string_lossy().fmt(f);
        }

        for char in self.chars() {
            f.write_char(char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for WideStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string_lossy(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::WideString, WideChar, WideStr};

    fn wide(string: &str) -> WideString {
        WideString::from(string)
    }

    #[test]
    fn from_bytes() {
        let string = WideStr::try_from_bytes(b"a\0b\0").unwrap();
        assert_eq!(*string, *"ab");
        assert!(WideStr::try_from_bytes(b"a\0b").is_none());
    }

    #[test]
    fn surrogate_pairs() {
        let string = wide("a😀");
        assert_eq!(string.len(), 3);
        assert_eq!(string.to_string(), "a😀");

        // An unpaired high surrogate is replaced
        let units = [WideChar::new(0xD83D), WideChar::new(u16::from(b'a'))];
        let unpaired = WideStr::from_units(&units);
        assert_eq!(unpaired.to_string_lossy(), "\u{FFFD}a");
    }

    #[test]
    fn to_ansi_lossy() {
        assert_eq!(wide("a€ā").to_ansi_lossy().as_bytes(), b"a\x80?");
    }

    #[test]
    fn ascii_case() {
        assert!(wide("0xFF").starts_with_ignore_ascii_case("0X"));
        assert!(!wide("0").starts_with_ignore_ascii_case("0x"));
        assert!(wide("Hello").eq_ignore_ascii_case(&wide("hELLO")));
        assert!(!wide("Hello").eq_ignore_ascii_case(&wide("hELL")));
    }

    #[test]
    fn parse_int() {
        assert_eq!(wide("123").parse_int(10), Some(123));
        assert_eq!(wide("ff").parse_int(16), Some(0xFF));
        assert_eq!(wide("1ā").parse_int(10), None);
    }

    #[test]
    fn indexing() {
        let string = wide("a string");
        assert_eq!(string[2..], *"string");
        assert_eq!(string.get_range(2..20), None);
    }
}
