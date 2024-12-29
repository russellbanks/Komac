use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::entry::condition::Condition;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};

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
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        if *version < InnoVersion(1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut directory = Self {
            name: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        Condition::load(reader, codepage, version)?;

        if *version >= InnoVersion(4, 0, 11) && *version < InnoVersion(4, 1, 0) {
            directory.permissions = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= InnoVersion(2, 0, 11) {
            directory.attributes = reader.read_u32::<LE>()?;
        }

        WindowsVersionRange::load(reader, version)?;

        if *version >= InnoVersion(4, 1, 0) {
            directory.permission = reader.read_i16::<LE>()?;
        } else {
            directory.permission = -1;
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
