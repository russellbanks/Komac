pub mod block;
mod compression;
mod decoder;
pub mod flags;

use std::io::{Error, ErrorKind, Read, Result, Seek};

use byteorder::ReadBytesExt;
use bzip2::read::BzDecoder;
pub use compression::Compression;
pub use decoder::Decoder;
use flate2::{Decompress, read::ZlibDecoder};
use liblzma::read::XzDecoder;
use tracing::debug;
use zerocopy::{FromBytes, I32, Immutable, KnownLayout, LE};

use super::FirstHeader;
use crate::analysis::installers::utils::LzmaStreamHeader;

const NSIS_MAX_INST_TYPES: u8 = 32;

#[derive(Clone, Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Header {
    #[doc(alias = "install_reg_rootkey")]
    install_registry_root_key: I32<LE>,

    #[doc(alias = "install_reg_key_ptr")]
    install_registry_key_ptr: I32<LE>,

    #[doc(alias = "install_reg_value_ptr")]
    install_registry_value_ptr: I32<LE>,

    #[doc(alias = "bg_color1")]
    background_color_1: I32<LE>,

    #[doc(alias = "bg_color2")]
    background_color_2: I32<LE>,

    #[doc(alias = "bg_textcolor")]
    background_text_color: I32<LE>,

    #[doc(alias = "lb_bg")]
    log_window_background_color: I32<LE>,

    #[doc(alias = "lb_fg")]
    log_window_foreground_color: I32<LE>,

    #[doc(alias = "langtable_size")]
    language_table_size: I32<LE>,

    #[doc(alias = "license_bg")]
    license_background_color: I32<LE>,

    #[doc(alias = "code_onInit")]
    code_on_init: I32<LE>,

    #[doc(alias = "code_onInstSuccess")]
    code_on_inst_success: I32<LE>,

    #[doc(alias = "code_onInstFailed")]
    code_on_inst_failed: I32<LE>,

    #[doc(alias = "code_onUserAbort")]
    code_on_user_abort: I32<LE>,

    #[doc(alias = "code_onGUIInit")]
    code_on_gui_init: I32<LE>,

    #[doc(alias = "code_onGUIEnd")]
    code_on_gui_end: I32<LE>,

    #[doc(alias = "code_onMouseOverSection")]
    code_on_mouse_over_section: I32<LE>,

    #[doc(alias = "code_onVerifyInstDir")]
    code_on_verify_install_dir: I32<LE>,

    #[doc(alias = "code_onSelChange")]
    code_on_sel_change: I32<LE>,

    #[doc(alias = "code_onRebootFailed")]
    code_on_reboot_failed: I32<LE>,

    install_types: [I32<LE>; NSIS_MAX_INST_TYPES as usize + 1],

    install_directory_ptr: I32<LE>,

    install_directory_auto_append: I32<LE>,

    str_uninstall_child: I32<LE>,

    str_uninstall_command: I32<LE>,

    /// Points to the path of `wininit.ini`.
    str_win_init: I32<LE>,
}

pub struct Decompressed<R: Read + Seek> {
    pub data: Vec<u8>,
    pub is_solid: bool,
    pub non_solid_start_offset: u32,
    pub compression: Compression,
    pub decoder: Decoder<R>,
}

fn is_lzma(data: &[u8]) -> Option<Compression> {
    fn is_lzma_header(data: &[u8]) -> bool {
        data.get(0..3) == Some([0x5D, 0, 0].as_slice())
            && data.get(5) == Some(&0)
            && data.get(6).is_some_and(|byte| byte & (1 << 7) == 0)
    }

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

const HEADER_SIGNATURE_SIZE: usize = 12;
const NON_SOLID_EXTRA_BYTES: usize = size_of::<u32>();
const IS_COMPRESSED_MASK: u32 = 1 << 31;

impl Header {
    /// <https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L5753>
    pub fn decompress<R: Read + Seek>(
        mut reader: R,
        first_header: &FirstHeader,
    ) -> Result<Decompressed<R>> {
        /*
          XX XX XX XX             XX XX XX XX == FirstHeader.HeaderSize, nonsolid, uncompressed
          5D 00 00 dd dd 00       solid LZMA
          00 5D 00 00 dd dd 00    solid LZMA, empty filter (there are no such archives)
          01 5D 00 00 dd dd 00    solid LZMA, BCJ filter   (only 7-Zip installer used that format)

          SS SS SS 80 00 5D 00 00 dd dd 00     non-solid LZMA, empty filter
          SS SS SS 80 01 5D 00 00 dd dd 00     non-solid LZMA, BCJ filter
          SS SS SS 80 01 tt         non-solid BZip (tt < 14)
          SS SS SS 80               non-solid Deflate

          01 tt         solid BZip (tt < 14)
          other         solid Deflate
        */

        // Read the signature
        let mut signature = [0; HEADER_SIGNATURE_SIZE];
        reader.read_exact(&mut signature)?;

        let mut compressed_header_size =
            u32::from_le_bytes(signature[..size_of::<u32>()].try_into().unwrap());
        let mut is_solid = true;

        debug!(?signature);

        let compression = if compressed_header_size == first_header.length_of_header() {
            is_solid = false;
            Compression::None
        } else if let Some(lzma_compression) = is_lzma(&signature) {
            lzma_compression
        } else if signature.get(3) == Some(&0x80) {
            is_solid = false;
            is_lzma(&signature[NON_SOLID_EXTRA_BYTES..]).map_or_else(
                || {
                    if is_bzip2(&signature[NON_SOLID_EXTRA_BYTES..]) {
                        Compression::BZip2
                    } else {
                        Compression::Zlib
                    }
                },
                |lzma_compression| lzma_compression,
            )
        } else if is_bzip2(&signature) {
            Compression::BZip2
        } else {
            Compression::Zlib
        };

        if is_solid {
            reader.seek_relative(-(HEADER_SIGNATURE_SIZE as i64))?;
        } else {
            reader.seek_relative(-((HEADER_SIGNATURE_SIZE - NON_SOLID_EXTRA_BYTES) as i64))?;
            compressed_header_size &= !IS_COMPRESSED_MASK;
        }

        debug!(?compression, is_solid, compressed_header_size);

        let mut decoder = match compression {
            Compression::Lzma(_) => {
                let stream = LzmaStreamHeader::from_reader(&mut reader)?;
                Decoder::Lzma(XzDecoder::new_stream(reader, stream))
            }
            Compression::BZip2 => Decoder::BZip2(BzDecoder::new(reader)),
            Compression::Zlib => Decoder::Zlib(ZlibDecoder::new_with_decompress(
                reader,
                Decompress::new(false),
            )),
            Compression::None => Decoder::None(reader),
        };

        if is_solid {
            let decompressed_header_size = decoder.read_u32::<byteorder::LE>()?;
            let expected_size = first_header.length_of_header();
            if decompressed_header_size != expected_size {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Decompressed header size ({decompressed_header_size}) did not equal size defined in first header ({expected_size})",
                    ),
                ));
            }
        }

        let mut decompressed_data = vec![0; first_header.length_of_header() as usize];
        decoder.read_exact(&mut decompressed_data)?;

        Ok(Decompressed {
            data: decompressed_data,
            is_solid,
            non_solid_start_offset: compressed_header_size,
            compression,
            decoder,
        })
    }

    #[inline]
    pub const fn language_table_size(&self) -> i32 {
        self.language_table_size.get()
    }

    #[inline]
    pub const fn install_directory_ptr(&self) -> i32 {
        self.install_directory_ptr.get()
    }

    #[inline]
    pub const fn has_install_directory(&self) -> bool {
        self.install_directory_ptr() != 0
    }
}
