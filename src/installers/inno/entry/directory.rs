use std::io::{Read, Result};

use bitflags::bitflags;
use byteorder::{LE, ReadBytesExt};
use encoding_rs::Encoding;

use crate::installers::inno::{
    encoding::InnoValue, entry::condition::Condition, version::InnoVersion,
    windows_version::WindowsVersionRange,
};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Directory {
    name: Option<String>,
    permissions: Option<String>,
    attributes: u32,
    /// Index into the permission entry list
    permission: i16,
    flags: DirectoryFlags,
}

impl Directory {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> Result<Self> {
        if *version < (1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut directory = Self {
            name: InnoValue::new_string(reader, codepage)?,
            permission: -1,
            ..Self::default()
        };

        Condition::from_reader(reader, codepage, version)?;

        if *version >= (4, 0, 11) && *version < (4, 1, 0) {
            directory.permissions = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (2, 0, 11) {
            directory.attributes = reader.read_u32::<LE>()?;
        }

        WindowsVersionRange::from_reader(reader, version)?;

        if *version >= (4, 1, 0) {
            directory.permission = reader.read_i16::<LE>()?;
        }

        directory.flags = DirectoryFlags::from_bits_retain(reader.read_u8()?);

        Ok(directory)
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct DirectoryFlags: u8 {
        const NEVER_UNINSTALL = 1 << 0;
        const DELETE_AFTER_INSTALL = 1 << 1;
        const ALWAYS_UNINSTALL = 1 << 2;
        const SET_NTFS_COMPRESSION = 1 << 3;
        const UNSET_NTFS_COMPRESSION = 1 << 4;
    }
}
