use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
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
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        let mut component = Self {
            name: InnoValue::new_string(reader, codepage)?,
            description: InnoValue::new_string(reader, codepage)?,
            types: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        if *version >= InnoVersion(4, 0, 1) {
            component.languages = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= InnoVersion(4, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 24))
        {
            component.check = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= InnoVersion(4, 0, 0) {
            component.extra_disk_space_required = reader.read_u64::<LE>()?;
        } else {
            component.extra_disk_space_required = u64::from(reader.read_u32::<LE>()?);
        }

        if *version >= InnoVersion(4, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(3, 0, 4))
        {
            component.used = reader.read_u8()? != 0;
        } else {
            component.used = true;
        }

        WindowsVersionRange::load(reader, version)?;

        component.flags = ComponentFlags::from_bits_retain(reader.read_u8()?);

        if *version >= InnoVersion(4, 0, 0) {
            component.size = reader.read_u64::<LE>()?;
        } else if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 24))
        {
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