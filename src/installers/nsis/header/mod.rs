pub mod block;
mod compression;
mod flags;

use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::block::{BlockHeader, BlockType};
use crate::installers::nsis::header::compression::Compression;
use crate::installers::nsis::header::flags::CommonHeaderFlags;
use crate::installers::utils::read_lzma_stream_header;
use byteorder::{ByteOrder, ReadBytesExt, LE};
use bzip2::read::BzDecoder;
use color_eyre::eyre::{bail, Result};
use flate2::read::DeflateDecoder;
use liblzma::read::XzDecoder;
use std::io::{Cursor, Read};
use strum::EnumCount;
use zerocopy::little_endian::{I32, U32};
use zerocopy::{FromBytes, Immutable, KnownLayout};

const NSIS_MAX_INST_TYPES: u8 = 32;

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Header {
    flags: CommonHeaderFlags,
    pub blocks: [BlockHeader; BlockType::COUNT],
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

impl Header {
    fn is_lzma(data: &[u8]) -> bool {
        let is_lzma_header = |d: &[u8]| {
            d.get(0..3) == Some([0x5D, 0, 0].as_slice())
                && d.get(5) == Some(&0)
                && d.get(6).map_or(false, |byte| byte & 1 << 7 == 0)
        };

        is_lzma_header(data) || (data.first() <= Some(&1) && is_lzma_header(&data[1..]))
    }

    fn is_bzip2(data: &[u8]) -> bool {
        const BZIP2_MAGIC: u8 = 0x31;

        data.first() == Some(&BZIP2_MAGIC) && data.get(1) < Some(&14)
    }

    pub fn decompress(data: &[u8], first_header: &FirstHeader) -> Result<Vec<u8>> {
        let signature = &data[..HEADER_SIGNATURE_SIZE as usize];
        let compressed_header_size = LE::read_u32(signature);
        let mut is_solid = true;

        let compression = if compressed_header_size == first_header.length_of_header.get() {
            is_solid = false;
            Compression::None
        } else if Self::is_lzma(signature) {
            Compression::Lzma
        } else if signature.get(3) == Some(&0x80) {
            is_solid = false;
            if Self::is_lzma(&signature[NON_SOLID_EXTRA_BYTES as usize..]) {
                Compression::Lzma
            } else if Self::is_bzip2(&signature[NON_SOLID_EXTRA_BYTES as usize..]) {
                Compression::BZip2
            } else {
                Compression::Zlib
            }
        } else if Self::is_bzip2(signature) {
            Compression::BZip2
        } else {
            Compression::Zlib
        };

        let mut reader = if is_solid {
            Cursor::new(data)
        } else {
            Cursor::new(&data[NON_SOLID_EXTRA_BYTES as usize..])
        };

        let mut decoder: Box<dyn Read> = match compression {
            Compression::Lzma => {
                let stream = read_lzma_stream_header(&mut reader)?;
                Box::new(XzDecoder::new_stream(reader, stream))
            }
            Compression::BZip2 => Box::new(BzDecoder::new(reader)),
            Compression::Zlib => Box::new(DeflateDecoder::new(reader)),
            Compression::None => Box::new(reader),
        };

        if is_solid && decoder.read_u32::<LE>()? != first_header.length_of_header.get() {
            bail!("Decompressed header size did not equal size defined in first header");
        }

        let mut uncompressed_data = vec![0; first_header.length_of_header.get() as usize];
        decoder.read_exact(&mut uncompressed_data)?;

        Ok(uncompressed_data)
    }
}
