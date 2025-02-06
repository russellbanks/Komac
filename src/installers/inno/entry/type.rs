use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::enum_value::enum_value::enum_value;
use crate::installers::inno::version::KnownVersion;
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Type {
    name: String,
    description: Option<String>,
    languages: Option<String>,
    check: Option<String>,
    is_custom: bool,
    setup: SetupType,
    size: u64,
}

impl Type {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        let mut r#type = Self {
            name: InnoValue::new_string(reader, codepage)?.unwrap_or_default(),
            description: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        if *version >= (4, 0, 1) {
            r#type.languages = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (1, 3, 24)) {
            r#type.check = InnoValue::new_string(reader, codepage)?;
        }

        WindowsVersionRange::from_reader(reader, version)?;

        let flags = TypeFlags::from_bits_retain(reader.read_u8()?);
        r#type.is_custom = flags.contains(TypeFlags::CUSTOM_SETUP_TYPE);

        if *version >= (4, 0, 3) {
            r#type.setup = enum_value!(reader, SetupType)?;
        }

        r#type.size = if *version >= (4, 0, 0) {
            reader.read_u64::<LE>()?
        } else {
            u64::from(reader.read_u32::<LE>()?)
        };

        Ok(r#type)
    }
}

#[expect(dead_code)]
#[derive(Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
enum SetupType {
    #[default]
    User,
    DefaultFull,
    DefaultCompact,
    DefaultCustom,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct TypeFlags: u8 {
        const CUSTOM_SETUP_TYPE = 1 << 0;
    }
}
