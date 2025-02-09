use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::entry::condition::Condition;
use crate::installers::inno::version::InnoVersion;
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Ini {
    file: String,
    section: Option<String>,
    key: Option<String>,
    value: Option<String>,
    flags: IniFlags,
}

impl Ini {
    const DEFAULT_FILE: &'static str = "{windows}/WIN.INI";

    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> Result<Self> {
        if *version < (1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut ini = Self {
            file: InnoValue::new_string(reader, codepage)?
                .unwrap_or_else(|| Self::DEFAULT_FILE.to_string()),
            section: InnoValue::new_string(reader, codepage)?,
            key: InnoValue::new_string(reader, codepage)?,
            value: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        Condition::from_reader(reader, codepage, version)?;

        WindowsVersionRange::from_reader(reader, version)?;

        ini.flags = IniFlags::from_bits_retain(reader.read_u8()?);

        Ok(ini)
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct IniFlags: u8 {
        const CREATE_KEY_IF_DOESNT_EXIST = 1 << 0;
        const UNINSTALL_DELETE_ENTRY = 1 << 1;
        const UNINSTALL_DELETE_ENTIRE_SECTION = 1 << 2;
        const UNINSTALL_DELETE_SECTION_IF_EMPTY = 1 << 3;
        const HAS_VALUE = 1 << 4;
    }
}
