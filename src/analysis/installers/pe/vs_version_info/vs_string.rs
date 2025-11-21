use std::{borrow::Cow, fmt, io};

use encoding_rs::{Encoding, UTF_16LE};

use super::VSHeader;
use crate::analysis::installers::pe::vs_version_info::vs_type::VSType;

/// Represents a [`String`](https://docs.microsoft.com/en-us/windows/win32/menurc/string-str) structure.
#[derive(Clone)]
pub struct VSString<'a> {
    header: VSHeader<'a>,
    value: Cow<'a, str>,
}

impl<'a> VSString<'a> {
    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        Self::read_from_with_encoding(data, UTF_16LE)
    }

    pub fn read_from_with_encoding(
        data: &'a [u8],
        encoding: &'static Encoding,
    ) -> io::Result<Self> {
        let header = VSHeader::read_from(data)?;

        let data = &data[header.end_offset..];

        let string_bytes = &data[..header.value_byte_length().saturating_sub(size_of::<u16>())];

        Ok(Self {
            header,
            value: encoding.decode_with_bom_removal(string_bytes).0,
        })
    }

    /// The length, in bytes, of this [`String`] structure.
    ///
    /// [`String`]: VSString
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    /// The size, in words, of the Value member.
    pub const fn value_length(&self) -> u16 {
        self.header.value_length()
    }

    #[must_use]
    #[inline]
    pub const fn r#type(&self) -> VSType {
        self.header.r#type()
    }

    #[must_use]
    #[inline]
    pub fn key(&self) -> &str {
        self.header.key()
    }

    #[must_use]
    #[inline]
    pub fn raw_value(&self) -> &str {
        &self.value
    }

    #[must_use]
    #[inline]
    pub fn value(&self) -> &str {
        self.value.trim()
    }
}

impl fmt::Display for VSString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key(), self.value())
    }
}

impl fmt::Debug for VSString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("String")
            .field("wLength", &self.length())
            .field("wValueLength", &self.value_length())
            .field("wType", &self.r#type())
            .field("szKey", &self.key())
            .field("Value", &self.value())
            .finish()
    }
}
