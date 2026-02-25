use std::{
    fmt,
    fmt::{Formatter, Pointer},
    io,
};

use super::{VSHeader, VSType, VSVar};

/// Represents the organization of data in a file-version resource. It contains version information
/// not dependent on a particular language and code page combination.
///
/// See <https://learn.microsoft.com/windows/win32/menurc/varfileinfo>
#[derive(Clone)]
pub struct VSVarFileInfo<'a> {
    header: VSHeader<'a>,
    children: Vec<VSVar<'a>>,
}

impl<'a> VSVarFileInfo<'a> {
    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        let header = VSHeader::read_from(data)?;

        let mut children = Vec::new();

        let mut offset = header.end_offset;

        while offset < usize::from(header.length()) {
            let child = VSVar::read_from(&data[offset..])?;

            offset += usize::from(child.length());
            offset = offset.next_multiple_of(size_of::<u32>());
            children.push(child);
        }

        Ok(Self { header, children })
    }

    /// Returns the length, in bytes, of the entire [`VarFileInfo`] block, including all [`Var`]
    /// children.
    ///
    /// [`VarFileInfo`]: VSVarFileInfo
    /// [`Var`]: VSVar
    #[must_use]
    #[inline]
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    #[must_use]
    #[inline]
    pub const fn value_length(&self) -> u16 {
        self.header.value_length()
    }

    #[must_use]
    #[inline]
    const fn r#type(&self) -> VSType {
        self.header.r#type()
    }

    #[must_use]
    #[inline]
    pub fn key(&self) -> &str {
        self.header.key()
    }

    /// Returns the Children as a slice of one or more [`Var`] structures.
    ///
    /// This is typically a list of languages that the application or DLL supports.
    ///
    /// [`Var`]: VSVar
    #[must_use]
    #[inline]
    pub const fn children(&self) -> &[VSVar<'_>] {
        self.children.as_slice()
    }
}

impl fmt::Debug for VSVarFileInfo<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarFileInfo")
            .field("wLength", &self.length())
            .field("wValueLength", &self.value_length())
            .field("wType", &self.r#type())
            .field("szKey", &self.key())
            .field("Children", &self.children())
            .finish()
    }
}
