use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::entry::condition::Condition;
use crate::installers::inno::enum_value::enum_value::enum_value;
use crate::installers::inno::flag_reader::read_flags::read_flags;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use crate::installers::utils::registry::RegRoot;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Registry {
    key: Option<String>,
    name: Option<String>,
    value: Option<String>,
    permissions: Option<String>,
    reg_root: RegRoot,
    permission: i16,
    r#type: RegistryType,
    flags: RegistryFlags,
}

impl Registry {
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        if *version < InnoVersion(1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut registry = Self {
            key: InnoValue::new_string(reader, codepage)?,
            name: InnoValue::new_string(reader, codepage)?,
            value: InnoValue::new_string(reader, codepage)?,
            permission: -1,
            ..Self::default()
        };

        Condition::load(reader, codepage, version)?;

        if *version >= InnoVersion(4, 0, 11) && *version < InnoVersion(4, 1, 0) {
            registry.permissions = InnoValue::new_string(reader, codepage)?;
        }

        WindowsVersionRange::load(reader, version)?;

        registry.reg_root = enum_value!(reader, RegRoot)?;

        if *version >= InnoVersion(4, 1, 0) {
            registry.permission = reader.read_i16::<LE>()?;
        };

        registry.r#type = enum_value!(reader, RegistryType)?;

        registry.flags = read_flags!(reader,
            [
                RegistryFlags::CREATE_VALUE_IF_DOESNT_EXIST,
                RegistryFlags::UNINSTALL_DELETE_VALUE,
                RegistryFlags::UNINSTALL_CLEAR_VALUE,
                RegistryFlags::UNINSTALL_DELETE_ENTIRE_KEY,
                RegistryFlags::UNINSTALL_DELETE_ENTIRE_KEY_IF_EMPTY,
            ],
            if *version >= InnoVersion(1, 2, 6) => RegistryFlags::PRESERVE_STRING_TYPE,
            if *version >= InnoVersion(1, 3, 9) => [
                RegistryFlags::DELETE_KEY,
                RegistryFlags::DELETE_VALUE
            ],
            if *version >= InnoVersion(1, 3, 12) => RegistryFlags::NO_ERROR,
            if *version >= InnoVersion(1, 3, 16) => RegistryFlags::DONT_CREATE_KEY,
            if *version >= InnoVersion(5, 1, 0) => [RegistryFlags::BITS_32, RegistryFlags::BITS_64]
        )?;

        Ok(registry)
    }
}

#[expect(dead_code)]
#[derive(Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
enum RegistryType {
    #[default]
    None,
    String,
    ExpandString,
    DWord,
    Binary,
    MultiString,
    QWord,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct RegistryFlags: u16 {
        const CREATE_VALUE_IF_DOESNT_EXIST = 1 << 0;
        const UNINSTALL_DELETE_VALUE = 1 << 1;
        const UNINSTALL_CLEAR_VALUE = 1 << 2;
        const UNINSTALL_DELETE_ENTIRE_KEY = 1 << 3;
        const UNINSTALL_DELETE_ENTIRE_KEY_IF_EMPTY = 1 << 4;
        const PRESERVE_STRING_TYPE = 1 << 5;
        const DELETE_KEY = 1 << 6;
        const DELETE_VALUE = 1 << 7;
        const NO_ERROR = 1 << 8;
        const DONT_CREATE_KEY = 1 << 9;
        const BITS_32 = 1 << 10;
        const BITS_64 = 1 << 11;
    }
}