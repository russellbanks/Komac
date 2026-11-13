use std::{
    borrow::{Borrow, Cow},
    fmt,
    ops::Deref,
};

use super::{AnsiChar, AnsiStr, str::encode_char};

/// An owned ANSI string, stored as raw Windows-1252 code units.
#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct AnsiString(Vec<AnsiChar>);

impl AnsiString {
    /// Creates a new empty `AnsiString`. This does not allocate.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new empty `AnsiString` with at least the specified capacity in bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Encodes a UTF-8 string as Windows-1252, replacing unrepresentable characters with `?`.
    #[must_use]
    pub fn from_str_lossy(string: &str) -> Self {
        let mut ansi = Self::with_capacity(string.len());
        ansi.push_str(string);
        ansi
    }

    /// Returns a borrowed [`AnsiStr`] view of the entire string.
    #[must_use]
    #[inline]
    pub fn as_ansi_str(&self) -> &AnsiStr {
        AnsiStr::from_bytes(&self.0)
    }

    /// Returns the raw Windows-1252 bytes of the string.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Converts the string into its underlying vector of Windows-1252 bytes.
    #[must_use]
    #[inline]
    pub fn into_bytes(self) -> Vec<AnsiChar> {
        self.0
    }

    /// Decodes the string from Windows-1252 into an owned UTF-8 [`String`].
    #[must_use]
    pub fn into_string_lossy(self) -> String {
        match self.as_ansi_str().to_string_lossy() {
            // All bytes were ASCII, so the bytes are valid UTF-8
            Cow::Borrowed(_) => String::from_utf8(self.0)
                .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned()),
            Cow::Owned(string) => string,
        }
    }

    /// Appends a single raw Windows-1252 byte to the end of the string.
    #[inline]
    pub fn push(&mut self, char: AnsiChar) {
        self.0.push(char);
    }

    /// Appends a [`char`], encoding it as Windows-1252 or `?` if it is not representable.
    #[inline]
    pub fn push_char(&mut self, char: char) {
        self.0.push(encode_char(char));
    }

    /// Appends an [`AnsiStr`] to the end of the string.
    #[inline]
    pub fn push_ansi_str(&mut self, string: &AnsiStr) {
        self.0.extend_from_slice(string.as_bytes());
    }

    /// Appends a UTF-8 string, encoding it as Windows-1252.
    pub fn push_str(&mut self, string: &str) {
        if string.is_ascii() {
            self.0.extend_from_slice(string.as_bytes());
        } else {
            self.0.extend(string.chars().map(encode_char));
        }
    }

    /// Removes the last byte from the string and returns it, or `None` if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<AnsiChar> {
        self.0.pop()
    }

    /// Shortens the string to `new_len` bytes. Does nothing if `new_len` is greater than the
    /// string's current length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    /// Removes all bytes from the string, keeping its allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Reserves capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Reserves capacity for exactly `additional` more bytes, without deliberately over-allocating.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Shrinks the capacity of the string as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Shrinks the capacity of the string to at least `min_capacity` bytes.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity);
    }

    /// Returns the number of bytes the string can hold without reallocating.
    #[must_use]
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl Deref for AnsiString {
    type Target = AnsiStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ansi_str()
    }
}

impl AsRef<AnsiStr> for AnsiString {
    #[inline]
    fn as_ref(&self) -> &AnsiStr {
        self.as_ansi_str()
    }
}

impl AsRef<[u8]> for AnsiString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Borrow<AnsiStr> for AnsiString {
    #[inline]
    fn borrow(&self) -> &AnsiStr {
        self.as_ansi_str()
    }
}

impl From<Vec<AnsiChar>> for AnsiString {
    #[inline]
    fn from(value: Vec<AnsiChar>) -> Self {
        Self(value)
    }
}

impl From<AnsiString> for Vec<AnsiChar> {
    #[inline]
    fn from(value: AnsiString) -> Self {
        value.0
    }
}

impl From<&AnsiStr> for AnsiString {
    #[inline]
    fn from(value: &AnsiStr) -> Self {
        value.to_owned()
    }
}

impl From<&[AnsiChar]> for AnsiString {
    #[inline]
    fn from(value: &[AnsiChar]) -> Self {
        Self(value.to_vec())
    }
}

impl From<&str> for AnsiString {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from_str_lossy(value)
    }
}

impl PartialEq<AnsiStr> for AnsiString {
    #[inline]
    fn eq(&self, other: &AnsiStr) -> bool {
        self.as_ansi_str() == other
    }
}

impl PartialEq<str> for AnsiString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self.as_ansi_str() == *other
    }
}

impl PartialEq<&str> for AnsiString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self.as_ansi_str() == **other
    }
}

impl FromIterator<AnsiChar> for AnsiString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = AnsiChar>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<AnsiChar> for AnsiString {
    #[inline]
    fn extend<T: IntoIterator<Item = AnsiChar>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<'a> Extend<&'a AnsiChar> for AnsiString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a AnsiChar>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().copied());
    }
}

impl fmt::Display for AnsiString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_ansi_str(), f)
    }
}

impl fmt::Debug for AnsiString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_ansi_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::AnsiString;

    #[test]
    fn push_str_utf8() {
        let mut string = AnsiString::new();
        string.push_str("a€");
        string.push_char('ā'); // Not representable in Windows-1252
        assert_eq!(string.as_bytes(), b"a\x80?");
        assert_eq!(string.into_string_lossy(), "a€?");
    }

    #[test]
    fn into_string_lossy_ascii() {
        assert_eq!(AnsiString::from("abc").into_string_lossy(), "abc");
    }
}
