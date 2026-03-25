use std::fmt;

use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U16, U32};

#[derive(Copy, Clone, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct ImageResourceDirectory {
    characteristics: U32<LittleEndian>,
    time_date_stamp: U32<LittleEndian>,
    major_version: U16<LittleEndian>,
    minor_version: U16<LittleEndian>,
    number_of_named_entries: U16<LittleEndian>,
    number_of_id_entries: U16<LittleEndian>,
}

impl ImageResourceDirectory {
    /// Returns the resource flags.
    ///
    /// This field is reserved for future use. It is currently set to zero.
    #[inline]
    pub const fn characteristics(&self) -> u32 {
        self.characteristics.get()
    }

    /// Returns the time that the resource data was created by the resource compiler.
    #[inline]
    pub const fn time_date_stamp(&self) -> u32 {
        self.time_date_stamp.get()
    }

    /// Returns the major version number, set by the user.
    #[inline]
    pub const fn major_version(&self) -> u16 {
        self.major_version.get()
    }

    /// Returns the minor version number, set by the user.
    #[inline]
    pub const fn minor_version(&self) -> u16 {
        self.major_version.get()
    }

    /// Returns the number of directory entries immediately following the table that use strings to
    /// identify Type, Name, or Language entries (depending on the level of the table).
    #[inline]
    pub const fn number_of_named_entries(&self) -> u16 {
        self.number_of_named_entries.get()
    }

    /// Returns the number of directory entries immediately following the Name entries that use
    /// numeric IDs for Type, Name, or Language entries.
    #[inline]
    pub const fn number_of_id_entries(&self) -> u16 {
        self.number_of_id_entries.get()
    }
}

impl fmt::Debug for ImageResourceDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IMAGE_RESOURCE_DIRECTORY")
            .field("Characteristics", &self.characteristics())
            .field("TimeDateStamp", &self.time_date_stamp())
            .field("MajorVersion", &self.major_version())
            .field("MinorVersion", &self.minor_version())
            .field("NumberOfNamedEntries", &self.number_of_named_entries())
            .field("NumberOfIdEntries", &self.number_of_id_entries())
            .finish()
    }
}
