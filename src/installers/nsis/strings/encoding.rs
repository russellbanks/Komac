use crate::installers::nsis::strings::code::NsCode;
use crate::installers::nsis::strings::lang::LangCode;
use crate::installers::nsis::strings::shell::Shell;
use crate::installers::nsis::strings::var::NsVar;
use crate::installers::nsis::version::NsisVersion;
use bon::builder;
use byteorder::{ReadBytesExt, LE};
use color_eyre::eyre::{Error, Result};
use std::io::{Cursor, Read};

const fn decode_number_from_char(mut char: u16) -> u16 {
    const ASCII_MASK: u16 = u16::from_le_bytes([u8::MAX >> 1; size_of::<u16>()]);

    // Convert each byte into ASCII value range (0x00 - 0x7F)
    char &= ASCII_MASK;
    let le_bytes = char.to_le_bytes();
    le_bytes[0] as u16 | ((le_bytes[1] as u16) << 7)
}

#[builder(finish_fn = get)]
pub fn nsis_string(
    strings_block: &[u8],
    relative_offset: u32,
    nsis_version: NsisVersion,
    unicode: bool,
) -> Result<String> {
    let mut nsis_string = String::new();
    resolve_nsis_string(
        &mut nsis_string,
        strings_block,
        relative_offset,
        nsis_version,
        unicode,
    )?;
    Ok(nsis_string)
}

/// Resolves a NSIS string given the strings block, a relative offset, and whether the string is
/// Unicode.
///
/// Instead of simply decoding a UTF-16LE or ANSI string by searching for a null byte, NSIS strings
/// contain special characters that must be handled. As a result, the string needs to be decoded
/// one character at a time.
#[expect(clippy::cast_possible_truncation)] // Truncating u16 `as u8` is intentional
fn resolve_nsis_string(
    buf: &mut String,
    strings_block: &[u8],
    relative_offset: u32,
    nsis_version: NsisVersion,
    unicode: bool,
) -> Result<()> {
    // Double the offset if the string is Unicode as each character will be 2 bytes
    let offset = relative_offset as usize * (usize::from(unicode) + 1);

    let mut reader = Cursor::new(&strings_block[offset..]);
    loop {
        let mut current = read_char(&mut reader, unicode)?;
        if current == 0 {
            break;
        }
        if NsCode::is_code(current as u8, nsis_version) {
            let mut next = read_char(&mut reader, unicode)?;
            if next == 0 {
                break;
            }
            if current != NsCode::Skip.get::<u16>(nsis_version) {
                let special_char = if unicode {
                    next
                } else {
                    u16::from_le_bytes([next as u8, read_char(&mut reader, unicode)? as u8])
                };
                if current == NsCode::Shell.get::<u16>(nsis_version) {
                    Shell::resolve(buf, strings_block, special_char, nsis_version, unicode)?;
                } else {
                    let index = if unicode {
                        next = decode_number_from_char(special_char);
                        next
                    } else {
                        decode_number_from_char(special_char)
                    };
                    if current == NsCode::Var.get::<u16>(nsis_version) {
                        NsVar::resolve(buf, u32::from(index), nsis_version);
                    } else if current == NsCode::Lang.get::<u16>(nsis_version) {
                        LangCode::resolve(buf, index);
                    }
                }
                continue;
            }
            current = next;
        }
        if let Some(character) = char::from_u32(u32::from(current)) {
            buf.push(character);
        }
    }
    Ok(())
}

/// Reads two bytes as little-endian if Unicode; otherwise, reads one byte and casts it to u16
fn read_char<R: Read>(reader: &mut R, unicode: bool) -> Result<u16> {
    if unicode {
        reader.read_u16::<LE>().map_err(Error::msg)
    } else {
        Ok(u16::from(reader.read_u8()?))
    }
}
