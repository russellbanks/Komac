use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::flag_reader::read_flags::read_flags;
use crate::installers::inno::version::InnoVersion;
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Task {
    name: Option<String>,
    description: Option<String>,
    group_description: Option<String>,
    components: Option<String>,
    languages: Option<String>,
    check: Option<String>,
    level: u32,
    used: bool,
    flags: TaskFlags,
}

impl Task {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> Result<Self> {
        let mut task = Self {
            name: InnoValue::new_string(reader, codepage)?,
            description: InnoValue::new_string(reader, codepage)?,
            group_description: InnoValue::new_string(reader, codepage)?,
            components: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        if *version >= (4, 0, 1) {
            task.languages = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (1, 3, 24)) {
            task.check = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (3, 0, 3)) {
            task.level = reader.read_u32::<LE>()?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (3, 0, 4)) {
            task.used = reader.read_u8()? != 0;
        } else {
            task.used = true;
        }

        WindowsVersionRange::from_reader(reader, version)?;

        task.flags = read_flags!(reader,
            [TaskFlags::EXCLUSIVE, TaskFlags::UNCHECKED],
            if *version >= (2, 0, 5) => TaskFlags::RESTART,
            if *version >= (2, 0, 6) => TaskFlags::CHECKED_ONCE,
            if *version >= (4, 2, 3) => TaskFlags::DONT_INHERIT_CHECK
        )?;

        Ok(task)
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct TaskFlags: u8 {
        const EXCLUSIVE = 1 << 0;
        const UNCHECKED = 1 << 1;
        const RESTART = 1 << 2;
        const CHECKED_ONCE = 1 << 3;
        const DONT_INHERIT_CHECK = 1 << 4;
    }
}
