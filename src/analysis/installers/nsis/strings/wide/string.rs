use std::{borrow::Borrow, fmt, ops::Deref};

use zerocopy::IntoBytes;

use super::{WideChar, WideStr};

/// An owned UTF-16LE string.
///
/// The string is not guaranteed to be valid UTF-16.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WideString(Vec<WideChar>);

impl WideString {
    /// Creates a new empty `WideString`.
    ///
    /// Given that the `WideString` is empty, this will not allocate any initial
    /// buffer. While that means that this initial operation is very
    /// inexpensive, it may cause excessive allocation later when you add data.
    /// If you have an idea of how much data the `WideString` will hold,
    /// consider the [`with_capacity`] method to prevent excessive
    /// re-allocation.
    ///
    /// [`with_capacity`]: WideString::with_capacity
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let s = WideString::new();
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new empty `WideString` with at least the specified capacity.
    ///
    /// `WideString`s have an internal buffer to hold their data. The capacity
    /// is the length of that buffer, and can be queried with the [`capacity`]
    /// method. This method creates an empty `WideString`, but one with an
    /// initial buffer that can hold at least `capacity` wide characters. This
    /// is useful when you may be appending a bunch of data to the `WideString`,
    /// reducing the number of reallocations it needs to do.
    ///
    /// [`capacity`]: WideString::capacity
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new`] method.
    ///
    /// [`new`]: WideString::new
    ///
    /// # Panics
    ///
    /// Panics if the capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut s = WideString::with_capacity(10);
    ///
    /// // The WideString contains no chars, even though it has capacity for more
    /// assert_eq!(s.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// let cap = s.capacity();
    /// for _ in 0..10 {
    ///     s.push_u16(b'a' as u16);
    /// }
    ///
    /// assert_eq!(s.capacity(), cap);
    ///
    /// // ...but this may make the wide string reallocate
    /// s.push_u16(b'a' as u16);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Encodes a UTF-8 string as UTF-16.
    #[must_use]
    pub fn encode_utf16(string: &str) -> Self {
        string.encode_utf16().map(WideChar::new).collect()
    }

    /// Returns a borrowed [`WideStr`] view of the entire string.
    #[inline]
    #[must_use]
    pub fn as_wide_str(&self) -> &WideStr {
        WideStr::from_units(&self.0)
    }

    /// Returns the UTF-16LE code units of the string.
    #[inline]
    #[must_use]
    pub const fn as_units(&self) -> &[WideChar] {
        self.0.as_slice()
    }

    /// Returns the raw UTF-16LE bytes of the string.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice().as_bytes()
    }

    /// Converts the string into its underlying vector of UTF-16LE code units.
    #[inline]
    #[must_use]
    pub fn into_units(self) -> Vec<WideChar> {
        self.0
    }

    /// Decodes the string into an owned UTF-8 [`String`], replacing unpaired surrogates with
    /// [`char::REPLACEMENT_CHARACTER`].
    #[inline]
    #[must_use]
    pub fn into_string_lossy(self) -> String {
        self.to_string_lossy()
    }

    /// Appends a single UTF-16LE code unit to the end of the string.
    #[inline]
    pub fn push(&mut self, value: WideChar) {
        self.0.push(value);
    }

    /// Appends a single native-endian UTF-16 code unit to the end of the string.
    #[inline]
    pub fn push_u16(&mut self, value: u16) {
        self.0.push(WideChar::new(value));
    }

    /// Appends a [`char`], encoding it as one or two UTF-16 code units.
    pub fn push_char(&mut self, value: char) {
        let mut buffer = [0; 2];

        for &unit in value.encode_utf16(&mut buffer).iter() {
            self.push_u16(unit);
        }
    }

    /// Appends a [`WideStr`] to the end of the string.
    #[inline]
    pub fn push_str(&mut self, string: &WideStr) {
        self.0.extend_from_slice(string.as_units());
    }

    /// Appends a UTF-8 string, encoding it as UTF-16.
    #[inline]
    pub fn push_str_utf8(&mut self, string: &str) {
        self.0.extend(string.encode_utf16().map(WideChar::new));
    }

    /// Removes the last code unit from the string and returns it, or `None` if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<WideChar> {
        self.0.pop()
    }

    /// Shortens the string to `new_len` code units. Does nothing if `new_len` is greater than the
    /// string's current length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    /// Removes all code units from the string, keeping its allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Reserves capacity for at least `additional` more code units.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Shrinks the capacity of the string as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Returns the number of code units the string can hold without reallocating.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl Deref for WideString {
    type Target = WideStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_wide_str()
    }
}

impl AsRef<WideStr> for WideString {
    #[inline]
    fn as_ref(&self) -> &WideStr {
        self.as_wide_str()
    }
}

impl Borrow<WideStr> for WideString {
    #[inline]
    fn borrow(&self) -> &WideStr {
        self.as_wide_str()
    }
}

impl From<Vec<WideChar>> for WideString {
    #[inline]
    fn from(value: Vec<WideChar>) -> Self {
        Self(value)
    }
}

impl From<WideString> for Vec<WideChar> {
    #[inline]
    fn from(value: WideString) -> Self {
        value.0
    }
}

impl From<&WideStr> for WideString {
    #[inline]
    fn from(value: &WideStr) -> Self {
        value.to_owned()
    }
}

impl From<&str> for WideString {
    #[inline]
    fn from(value: &str) -> Self {
        Self::encode_utf16(value)
    }
}

impl PartialEq<WideStr> for WideString {
    #[inline]
    fn eq(&self, other: &WideStr) -> bool {
        self.as_wide_str() == other
    }
}

impl PartialEq<str> for WideString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self.as_wide_str() == *other
    }
}

impl PartialEq<&str> for WideString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self.as_wide_str() == **other
    }
}

impl FromIterator<WideChar> for WideString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = WideChar>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Extend<WideChar> for WideString {
    #[inline]
    fn extend<T: IntoIterator<Item = WideChar>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<'a> Extend<&'a WideChar> for WideString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a WideChar>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().copied());
    }
}

impl fmt::Display for WideString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_wide_str(), f)
    }
}

impl fmt::Debug for WideString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_wide_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::WideString;

    #[test]
    fn push() {
        let mut string = WideString::new();
        string.push_char('😀');
        string.push_str_utf8("ab");
        string.push_u16(u16::from(b'c'));
        assert_eq!(string.len(), 5);
        assert_eq!(string, "😀abc");
        string.truncate(2);
        assert_eq!(string.into_string_lossy(), "😀");
    }
}
