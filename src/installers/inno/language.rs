use crate::installers::inno::encoding::encoded_string;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use byteorder::{ReadBytesExt, LE};
use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};
use std::io::{Read, Result};

#[derive(Debug, Default)]
pub struct LanguageEntry {
    name: String,
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
    codepage: Option<&'static Encoding>,
    dialog_font_size: u32,
    dialog_font_standard_height: u32,
    title_font_size: u32,
    welcome_font_size: u32,
    copyright_font_size: u32,
    right_to_left: bool,
}

impl LanguageEntry {
    pub fn load<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<Self> {
        let mut entry = Self::default();
        if *version >= InnoVersion(4, 0, 0) {
            entry.name =
                encoded_string(reader, UTF_16LE)?.unwrap_or_else(|| String::from("default"));
        }

        entry.language_name = encoded_string(reader, UTF_16LE)?;
        entry.dialog_font = encoded_string(reader, UTF_16LE)?;
        entry.title_font = encoded_string(reader, UTF_16LE)?;
        entry.welcome_font = encoded_string(reader, UTF_16LE)?;
        entry.copyright_font = encoded_string(reader, UTF_16LE)?;

        if *version >= InnoVersion(4, 0, 0) {
            entry.data = encoded_string(reader, UTF_16LE)?;
        }

        if *version >= InnoVersion(4, 0, 1) {
            entry.license_text = encoded_string(reader, UTF_16LE)?;
            entry.info_before = encoded_string(reader, UTF_16LE)?;
            entry.info_after = encoded_string(reader, UTF_16LE)?;
        }

        entry.language_id = reader.read_u32::<LE>()?;

        if *version < InnoVersion(4, 2, 2) {
            entry.codepage = u16::try_from(entry.language_id)
                .ok()
                .and_then(codepage::to_encoding);
        } else if !version.is_unicode() {
            let codepage = reader.read_u32::<LE>()?;
            entry.codepage = if codepage != 0 {
                u16::try_from(codepage).ok().and_then(codepage::to_encoding)
            } else {
                Some(WINDOWS_1252)
            };
        } else {
            if *version < InnoVersion(5, 3, 0) {
                reader.read_u32::<LE>()?;
            }
            entry.codepage = Some(UTF_16LE);
        }

        entry.dialog_font_size = reader.read_u32::<LE>()?;

        if *version < InnoVersion(4, 1, 0) {
            entry.dialog_font_standard_height = reader.read_u32::<LE>()?;
        }

        entry.title_font_size = reader.read_u32::<LE>()?;
        entry.welcome_font_size = reader.read_u32::<LE>()?;
        entry.copyright_font_size = reader.read_u32::<LE>()?;

        if *version >= InnoVersion(5, 2, 3) {
            entry.right_to_left = reader.read_u8()? != 0;
        }

        Ok(entry)
    }
}
