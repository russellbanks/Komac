use std::fmt;

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U16};

#[repr(C)]
#[derive(Copy, Clone, FromBytes, Immutable, KnownLayout)]
pub struct VarDword {
    lang_id: U16<LE>,
    codepage: U16<LE>,
}

impl VarDword {
    /// Returns the Microsoft language identifier.
    #[must_use]
    #[inline]
    pub const fn lang_id(self) -> u16 {
        self.lang_id.get()
    }

    /// Returns the IBM code page number.
    #[must_use]
    #[inline]
    pub const fn codepage(self) -> u16 {
        self.codepage.get()
    }
}

impl fmt::Debug for VarDword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarDword")
            .field("LangID", &self.lang_id())
            .field("Codepage", &self.codepage())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::VarDword;

    #[test]
    fn size() {
        assert_eq!(size_of::<VarDword>(), size_of::<u16>() * 2);
    }
}
