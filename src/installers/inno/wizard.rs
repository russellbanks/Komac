use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::header::enums::Compression;
use crate::installers::inno::header::flags::HeaderFlags;
use crate::installers::inno::header::Header;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Result};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Wizard {
    images: Vec<Vec<u8>>,
    small_images: Vec<Vec<u8>>,
    decompressor_dll: Option<Vec<u8>>,
    decrypt_dll: Option<Vec<u8>>,
}

impl Wizard {
    pub fn load<R: Read>(reader: &mut R, version: &KnownVersion, header: &Header) -> Result<Self> {
        let mut wizard = Self {
            images: Self::load_images(reader, version)?,
            ..Self::default()
        };

        if *version >= InnoVersion(2, 0, 0) || version.is_isx() {
            wizard.small_images = Self::load_images(reader, version)?;
        }

        if header.compression == Compression::BZip2
            || (header.compression == Compression::LZMA1 && *version == InnoVersion(4, 1, 5))
            || (header.compression == Compression::Zlib && *version >= InnoVersion(4, 2, 6))
        {
            wizard.decompressor_dll = InnoValue::new_raw(reader)?;
        }

        if header.flags.contains(HeaderFlags::ENCRYPTION_USED) {
            wizard.decrypt_dll = InnoValue::new_raw(reader)?;
        }

        Ok(wizard)
    }

    fn load_images<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<Vec<Vec<u8>>> {
        let count = if *version >= InnoVersion(5, 6, 0) {
            reader.read_u32::<LE>()?
        } else {
            1
        };

        let mut images = (0..count)
            .filter_map(|_| InnoValue::new_raw(reader).transpose())
            .collect::<Result<Vec<_>>>()?;

        if *version < InnoVersion(5, 6, 0) && images.first().is_some_and(Vec::is_empty) {
            images.clear();
        }

        Ok(images)
    }
}
