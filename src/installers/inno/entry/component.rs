use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::version::InnoVersion;
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{LE, ReadBytesExt};
use encoding_rs::Encoding;
use std::io::{Read, Result};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Component {
    name: Option<String>,
    description: Option<String>,
    types: Option<String>,
    languages: Option<String>,
    check: Option<String>,
    extra_disk_space_required: u64,
    level: u32,
    used: bool,
    flags: ComponentFlags,
    size: u64,
}

impl Component {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> Result<Self> {
        let mut component = Self {
            name: InnoValue::new_string(reader, codepage)?,
            description: InnoValue::new_string(reader, codepage)?,
            types: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        if *version >= (4, 0, 1) {
            component.languages = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (1, 3, 24)) {
            component.check = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) {
            component.extra_disk_space_required = reader.read_u64::<LE>()?;
        } else {
            component.extra_disk_space_required = u64::from(reader.read_u32::<LE>()?);
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (3, 0, 3)) {
            component.level = reader.read_u32::<LE>()?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (3, 0, 4)) {
            component.used = reader.read_u8()? != 0;
        } else {
            component.used = true;
        }

        WindowsVersionRange::from_reader(reader, version)?;

        component.flags = ComponentFlags::from_bits_retain(reader.read_u8()?);

        if *version >= (4, 0, 0) {
            component.size = reader.read_u64::<LE>()?;
        } else if *version >= (2, 0, 0) || (version.is_isx() && *version >= (1, 3, 24)) {
            component.size = u64::from(reader.read_u32::<LE>()?);
        }

        Ok(component)
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct ComponentFlags: u8 {
        const FIXED = 1 << 0;
        const RESTART = 1 << 1;
        const DISABLE_NO_UNINSTALL_WARNING = 1 << 2;
        const EXCLUSIVE = 1 << 3;
        const DONT_INHERIT_CHECK = 1 << 4;
    }
}
