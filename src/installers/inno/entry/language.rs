use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::version::KnownVersion;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};
use std::io::{Read, Result};

#[derive(Debug)]
pub struct Language {
    name: Option<String>,
    language_name: Option<String>,
    dialog_font: Option<String>,
    title_font: Option<String>,
    welcome_font: Option<String>,
    copyright_font: Option<String>,
    data: Option<String>,
    license_text: Option<String>,
    info_before: Option<String>,
    info_after: Option<String>,
    pub language_id: u32,
    pub codepage: &'static Encoding,
    dialog_font_size: u32,
    dialog_font_standard_height: u32,
    title_font_size: u32,
    welcome_font_size: u32,
    copyright_font_size: u32,
    right_to_left: bool,
}

impl Language {
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        let mut entry = Self::default();

        if *version >= (4, 0, 0) {
            entry.name = InnoValue::new_string(reader, codepage)?;
        }

        entry.language_name = InnoValue::new_string(reader, codepage)?;
        entry.dialog_font = InnoValue::new_string(reader, codepage)?;
        entry.title_font = InnoValue::new_string(reader, codepage)?;
        entry.welcome_font = InnoValue::new_string(reader, codepage)?;
        entry.copyright_font = InnoValue::new_string(reader, codepage)?;

        if *version >= (4, 0, 0) {
            entry.data = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 1) {
            entry.license_text = InnoValue::new_string(reader, codepage)?;
            entry.info_before = InnoValue::new_string(reader, codepage)?;
            entry.info_after = InnoValue::new_string(reader, codepage)?;
        }

        entry.language_id = reader.read_u32::<LE>()?;

        if *version < (4, 2, 2) {
            entry.codepage = u16::try_from(entry.language_id)
                .ok()
                .and_then(codepage::to_encoding)
                .unwrap_or(WINDOWS_1252);
        } else if !version.is_unicode() {
            let codepage = reader.read_u32::<LE>()?;
            entry.codepage = (codepage != 0)
                .then(|| u16::try_from(codepage).ok().and_then(codepage::to_encoding))
                .flatten()
                .unwrap_or(WINDOWS_1252);
        } else {
            if *version < (5, 3, 0) {
                reader.read_u32::<LE>()?;
            }
            entry.codepage = UTF_16LE;
        }

        entry.dialog_font_size = reader.read_u32::<LE>()?;

        if *version < (4, 1, 0) {
            entry.dialog_font_standard_height = reader.read_u32::<LE>()?;
        }

        entry.title_font_size = reader.read_u32::<LE>()?;
        entry.welcome_font_size = reader.read_u32::<LE>()?;
        entry.copyright_font_size = reader.read_u32::<LE>()?;

        if *version >= (5, 2, 3) {
            entry.right_to_left = reader.read_u8()? != 0;
        }

        Ok(entry)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self {
            name: None,
            language_name: None,
            dialog_font: None,
            title_font: None,
            welcome_font: None,
            copyright_font: None,
            data: None,
            license_text: None,
            info_before: None,
            info_after: None,
            language_id: 0,
            codepage: WINDOWS_1252,
            dialog_font_size: 0,
            dialog_font_standard_height: 0,
            title_font_size: 0,
            welcome_font_size: 0,
            copyright_font_size: 0,
            right_to_left: false,
        }
    }
}
