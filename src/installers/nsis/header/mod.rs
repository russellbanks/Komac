pub mod block;
pub mod compression;
pub mod flags;

use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::compression::Compression;
use crate::installers::utils::read_lzma_stream_header;
use byteorder::{ByteOrder, ReadBytesExt, LE};
use bzip2::read::BzDecoder;
use flate2::read::DeflateDecoder;
use liblzma::read::XzDecoder;
use std::io::{Error, ErrorKind, Read, Result};
use zerocopy::little_endian::{I32, U32};
use zerocopy::{FromBytes, Immutable, KnownLayout};

const NSIS_MAX_INST_TYPES: u8 = 32;

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Header {
    install_reg_rootkey: U32,
    install_rek_key_ptr: U32,
    install_reg_value_ptr: U32,
    bg_color1: U32,
    bg_color2: U32,
    bg_textcolor: U32,
    lb_bg: U32,
    lb_fg: U32,
    pub langtable_size: U32,
    license_bg: U32,
    code_on_init: I32,
    code_on_inst_success: I32,
    code_on_inst_failed: I32,
    code_on_user_abort: I32,
    code_on_gui_init: I32,
    code_on_gui_end: I32,
    code_on_mouse_over_section: I32,
    code_on_verify_install_dir: I32,
    code_on_sel_change: I32,
    code_on_reboot_failed: I32,
    install_types: [I32; NSIS_MAX_INST_TYPES as usize + 1],
    pub install_directory_ptr: U32,
    install_directory_auto_append: U32,
    str_uninstall_child: I32,
    str_uninstall_command: I32,
    str_win_init: I32,
}

const HEADER_SIGNATURE_SIZE: u8 = 12;
const NON_SOLID_EXTRA_BYTES: u8 = 1 << 2;

const IS_COMPRESSED_MASK: u32 = 1 << 31;

pub struct Decompressed<'data> {
    pub decompressed_data: Vec<u8>,
    pub is_solid: bool,
    pub non_solid_start_offset: u32,
    pub compression: Compression,
    pub decoder: Box<dyn Read + 'data>,
}

fn is_lzma(data: &[u8]) -> Option<Compression> {
    let is_lzma_header = |d: &[u8]| {
        d.get(0..3) == Some([0x5D, 0, 0].as_slice())
            && d.get(5) == Some(&0)
            && d.get(6).is_some_and(|byte| byte & 1 << 7 == 0)
    };

    if is_lzma_header(data) {
        Some(Compression::Lzma(false))
    } else if data.first() <= Some(&1) && is_lzma_header(&data[1..]) {
        Some(Compression::Lzma(true))
    } else {
        None
    }
}

fn is_bzip2(data: &[u8]) -> bool {
    const BZIP2_MAGIC: u8 = 0x31;

    data.first() == Some(&BZIP2_MAGIC) && data.get(1) < Some(&14)
}

impl Header {
    pub fn decompress<'data>(
        data: &'data [u8],
        first_header: &'data FirstHeader,
    ) -> Result<Decompressed<'data>> {
        let signature = &data[..HEADER_SIGNATURE_SIZE as usize];
        let mut compressed_header_size = LE::read_u32(signature);
        let mut is_solid = true;

        let compression = if compressed_header_size == first_header.length_of_header.get() {
            is_solid = false;
            Compression::None
        } else if let Some(lzma_compression) = is_lzma(signature) {
            lzma_compression
        } else if signature.get(3) == Some(&0x80) {
            is_solid = false;
            is_lzma(&signature[NON_SOLID_EXTRA_BYTES as usize..]).map_or_else(
                || {
                    if is_bzip2(&signature[NON_SOLID_EXTRA_BYTES as usize..]) {
                        Compression::BZip2
                    } else {
                        Compression::Zlib
                    }
                },
                |lzma_compression| lzma_compression,
            )
        } else if is_bzip2(signature) {
            Compression::BZip2
        } else {
            Compression::Zlib
        };

        let mut data = if is_solid {
            data
        } else {
            compressed_header_size &= !IS_COMPRESSED_MASK;
            &data[NON_SOLID_EXTRA_BYTES as usize..]
        };

        let mut decoder: Box<dyn Read> = match compression {
            Compression::Lzma(_) => {
                let stream = read_lzma_stream_header(&mut data)?;
                Box::new(XzDecoder::new_stream(data, stream))
            }
            Compression::BZip2 => Box::new(BzDecoder::new(data)),
            Compression::Zlib => Box::new(DeflateDecoder::new(data)),
            Compression::None => Box::new(data),
        };

        if is_solid && decoder.read_u32::<LE>()? != first_header.length_of_header.get() {
            Error::new(
                ErrorKind::InvalidData,
                "Decompressed header size did not equal size defined in first header",
            );
        }

        let mut decompressed_data = vec![0; first_header.length_of_header.get() as usize];
        decoder.read_exact(&mut decompressed_data)?;

        Ok(Decompressed {
            decompressed_data,
            is_solid,
            non_solid_start_offset: compressed_header_size,
            compression,
            decoder,
        })
    }
}
