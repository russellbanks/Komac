use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::entry::condition::Condition;
use crate::installers::inno::enum_value::enum_value::enum_value;
use crate::installers::inno::flag_reader::read_flags::read_flags;
use crate::installers::inno::version::KnownVersion;
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Icon {
    name: Option<String>,
    filename: Option<String>,
    parameters: Option<String>,
    working_directory: Option<String>,
    file: Option<String>,
    comment: Option<String>,
    app_user_model_id: Option<String>,
    app_user_model_toast_activator_clsid: String,
    index: i32,
    show_command: i32,
    close_on_exit: CloseSetting,
    hotkey: u16,
    flags: IconFlags,
}

impl Icon {
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        if *version < (1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut icon = Self {
            name: InnoValue::new_string(reader, codepage)?,
            filename: InnoValue::new_string(reader, codepage)?,
            parameters: InnoValue::new_string(reader, codepage)?,
            working_directory: InnoValue::new_string(reader, codepage)?,
            file: InnoValue::new_string(reader, codepage)?,
            comment: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        Condition::load(reader, codepage, version)?;

        if *version >= (5, 3, 5) {
            icon.app_user_model_id = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (6, 1, 0) {
            let mut buf = [0; 16];
            reader.read_exact(&mut buf)?;
            icon.app_user_model_toast_activator_clsid = codepage.decode(&buf).0.into_owned();
        }

        WindowsVersionRange::load(reader, version)?;

        icon.index = reader.read_i32::<LE>()?;

        icon.show_command = if *version >= (1, 3, 24) {
            reader.read_i32::<LE>()?
        } else {
            1
        };

        if *version >= (1, 3, 15) {
            icon.close_on_exit = enum_value!(reader, CloseSetting)?;
        }

        if *version >= (2, 0, 7) {
            icon.hotkey = reader.read_u16::<LE>()?;
        }

        icon.flags = read_flags!(reader,
            IconFlags::NEVER_UNINSTALL,
            if *version < (1, 3, 26) => IconFlags::RUN_MINIMIZED,
            [IconFlags::CREATE_ONLY_IF_FILE_EXISTS, IconFlags::USE_APP_PATHS],
            if *version >= (5, 0, 3) && *version < (6, 3, 0) => IconFlags::FOLDER_SHORTCUT,
            if *version >= (5, 4, 2) => IconFlags::EXCLUDE_FROM_SHOW_IN_NEW_INSTALL,
            if *version >= (5, 5, 0) => IconFlags::PREVENT_PINNING,
            if *version >= (6, 1, 0) => IconFlags::HAS_APP_USER_MODEL_TOAST_ACTIVATOR_CLSID
        )?;

        Ok(icon)
    }
}

#[expect(dead_code)]
#[derive(Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
enum CloseSetting {
    #[default]
    NoSetting,
    CloseOnExit,
    DontCloseOnExit,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct IconFlags: u8 {
        const NEVER_UNINSTALL = 1 << 0;
        const CREATE_ONLY_IF_FILE_EXISTS = 1 << 1;
        const USE_APP_PATHS = 1 << 2;
        const FOLDER_SHORTCUT = 1 << 3;
        const EXCLUDE_FROM_SHOW_IN_NEW_INSTALL = 1 << 4;
        const PREVENT_PINNING = 1 << 5;
        const HAS_APP_USER_MODEL_TOAST_ACTIVATOR_CLSID = 1 << 6;
        const RUN_MINIMIZED = 1 << 7;
    }
}
