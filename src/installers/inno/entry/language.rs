use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::version::InnoVersion;
use byteorder::{LE, ReadBytesExt};
use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};
use std::io::{Read, Result};

#[derive(Debug)]
pub struct Language {
    internal_name: Option<String>,
    name: Option<String>,
    dialog_font: Option<String>,
    title_font: Option<String>,
    welcome_font: Option<String>,
    copyright_font: Option<String>,
    data: Option<String>,
    license_text: Option<String>,
    info_before: Option<String>,
    info_after: Option<String>,
    pub id: u32,
    pub codepage: &'static Encoding,
    dialog_font_size: u32,
    dialog_font_standard_height: u32,
    title_font_size: u32,
    welcome_font_size: u32,
    copyright_font_size: u32,
    right_to_left: bool,
}

impl Language {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> Result<Self> {
        let mut language = Self::default();

        if *version >= (4, 0, 0) {
            language.internal_name = InnoValue::new_string(reader, codepage)?;
        }

        language.name = InnoValue::new_string(reader, codepage)?;
        language.dialog_font = InnoValue::new_string(reader, codepage)?;
        language.title_font = InnoValue::new_string(reader, codepage)?;
        language.welcome_font = InnoValue::new_string(reader, codepage)?;
        language.copyright_font = InnoValue::new_string(reader, codepage)?;

        if *version >= (4, 0, 0) {
            language.data = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 1) {
            language.license_text = InnoValue::new_string(reader, codepage)?;
            language.info_before = InnoValue::new_string(reader, codepage)?;
            language.info_after = InnoValue::new_string(reader, codepage)?;
        }

        language.id = reader.read_u32::<LE>()?;

        if *version < (4, 2, 2) {
            language.codepage = u16::try_from(language.id)
                .ok()
                .and_then(codepage::to_encoding)
                .unwrap_or(WINDOWS_1252);
        } else if !version.is_unicode() {
            let codepage = reader.read_u32::<LE>()?;
            language.codepage = (codepage != 0)
                .then(|| u16::try_from(codepage).ok().and_then(codepage::to_encoding))
                .flatten()
                .unwrap_or(WINDOWS_1252);
        } else {
            if *version < (5, 3, 0) {
                reader.read_u32::<LE>()?;
            }
            language.codepage = UTF_16LE;
        }

        language.dialog_font_size = reader.read_u32::<LE>()?;

        if *version < (4, 1, 0) {
            language.dialog_font_standard_height = reader.read_u32::<LE>()?;
        }

        language.title_font_size = reader.read_u32::<LE>()?;
        language.welcome_font_size = reader.read_u32::<LE>()?;
        language.copyright_font_size = reader.read_u32::<LE>()?;

        if *version >= (5, 2, 3) {
            language.right_to_left = reader.read_u8()? != 0;
        }

        Ok(language)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self {
            internal_name: None,
            name: None,
            dialog_font: None,
            title_font: None,
            welcome_font: None,
            copyright_font: None,
            data: None,
            license_text: None,
            info_before: None,
            info_after: None,
            id: 0,
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
