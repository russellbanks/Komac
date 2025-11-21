use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U16, U32};

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct ImageResourceDirectory {
    characteristics: U32<LittleEndian>,
    time_date_stamp: U32<LittleEndian>,
    major_version: U16<LittleEndian>,
    minor_version: U16<LittleEndian>,
    number_of_name_entries: U16<LittleEndian>,
    number_of_id_entries: U16<LittleEndian>,
}

impl ImageResourceDirectory {
    #[inline]
    pub const fn characteristics(&self) -> u32 {
        self.characteristics.get()
    }

    #[inline]
    pub const fn time_date_stamp(&self) -> u32 {
        self.time_date_stamp.get()
    }

    #[inline]
    pub const fn major_version(&self) -> u16 {
        self.major_version.get()
    }

    #[inline]
    pub const fn minor_version(&self) -> u16 {
        self.major_version.get()
    }

    #[inline]
    pub const fn number_of_name_entries(&self) -> u16 {
        self.number_of_name_entries.get()
    }

    #[inline]
    pub const fn number_of_id_entries(&self) -> u16 {
        self.number_of_id_entries.get()
    }
}
