use super::super::{AnsiChar, WideChar, decode_char, encode_char};

/// A single NSIS code unit, which is a byte in ANSI installers and a UTF-16 code unit in Unicode
/// installers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NsisChar {
    /// A Windows-1252 byte from an ANSI installer.
    Ansi(AnsiChar),
    /// A UTF-16LE code unit from a Unicode installer.
    Wide(WideChar),
}

impl NsisChar {
    /// Returns `true` if this is an ANSI code unit.
    #[must_use]
    #[inline]
    pub const fn is_ansi(self) -> bool {
        matches!(self, Self::Ansi(_))
    }

    /// Returns `true` if this is a UTF-16 code unit.
    #[must_use]
    #[inline]
    pub const fn is_wide(self) -> bool {
        matches!(self, Self::Wide(_))
    }

    /// Converts the code unit into an ANSI byte, encoding it as Windows-1252 or `?` if it is not
    /// representable.
    #[must_use]
    pub fn to_ansi(self) -> AnsiChar {
        match self {
            Self::Ansi(char) => char,
            Self::Wide(char) => char::from_u32(u32::from(char.get())).map_or(b'?', encode_char),
        }
    }

    /// Converts the code unit into a UTF-16 code unit, decoding ANSI bytes as Windows-1252.
    #[must_use]
    pub fn to_wide(self) -> WideChar {
        match self {
            Self::Ansi(char) => {
                // Every Windows-1252 character is in the Basic Multilingual Plane
                let mut buf = [0; 1];
                decode_char(char).encode_utf16(&mut buf);
                WideChar::new(buf[0])
            }
            Self::Wide(char) => char,
        }
    }
}

impl From<AnsiChar> for NsisChar {
    #[inline]
    fn from(value: AnsiChar) -> Self {
        Self::Ansi(value)
    }
}

impl From<WideChar> for NsisChar {
    #[inline]
    fn from(value: WideChar) -> Self {
        Self::Wide(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::WideChar, NsisChar};

    #[test]
    fn conversion() {
        assert_eq!(NsisChar::Ansi(0x80).to_wide(), WideChar::new(0x20AC));
        assert_eq!(NsisChar::Wide(WideChar::new(0x20AC)).to_ansi(), 0x80);
        assert_eq!(NsisChar::Wide(WideChar::new(0x0101)).to_ansi(), b'?');
        assert_eq!(NsisChar::Wide(WideChar::new(0xD83D)).to_ansi(), b'?');
    }
}
