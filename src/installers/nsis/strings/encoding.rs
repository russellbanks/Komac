use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::code::NsCode;
use crate::installers::nsis::strings::lang::LangCode;
use crate::installers::nsis::strings::shell::Shell;
use crate::installers::nsis::strings::var::NsVar;
use crate::installers::nsis::version::NsisVersion;
use byteorder::{ByteOrder, LE};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use itertools::Either;
use std::borrow::Cow;

const fn decode_number_from_char(mut char: u16) -> u16 {
    const ASCII_MASK: u16 = u16::from_le_bytes([u8::MAX >> 1; size_of::<u16>()]);

    // Convert each byte into ASCII value range (0x00 - 0x7F)
    char &= ASCII_MASK;
    let le_bytes = char.to_le_bytes();
    le_bytes[0] as u16 | ((le_bytes[1] as u16) << 7)
}

#[expect(clippy::cast_possible_truncation)] // Truncating u16 `as u8` is intentional
pub fn nsis_string<'str_block>(
    strings_block: &'str_block [u8],
    relative_offset: u32,
    entries: &[Entry],
    nsis_version: NsisVersion,
) -> Cow<'str_block, str> {
    // The strings block starts with a UTF-16 null byte if it is Unicode
    let unicode = &strings_block[..size_of::<u16>()] == b"\0\0";

    // Double the offset if the string is Unicode as each character will be 2 bytes
    let offset = relative_offset as usize * (usize::from(unicode) + 1);

    // Get the index of the null byte at the end of the string
    let string_end_index = if unicode {
        strings_block[offset..]
            .chunks_exact(size_of::<u16>())
            .position(|chunk| chunk == b"\0\0")
            .map(|index| index * size_of::<u16>())
    } else {
        memchr::memchr(0, &strings_block[offset..])
    }
    .unwrap_or(strings_block.len());

    let string_bytes = &strings_block[offset..offset + string_end_index];

    // Check whether the string contains any special characters that need to be decoded
    let contains_code = if unicode {
        string_bytes
            .chunks_exact(size_of::<u16>())
            .map(LE::read_u16)
            .any(|char| u8::try_from(char).is_ok_and(|code| NsCode::is_code(code, nsis_version)))
    } else {
        string_bytes
            .iter()
            .any(|&char| NsCode::is_code(char, nsis_version))
    };

    // If the string doesn't have any special characters, we can just decode it normally
    if !contains_code {
        let encoding = if unicode { UTF_16LE } else { WINDOWS_1252 };
        return encoding.decode_without_bom_handling(string_bytes).0;
    }

    // Create an iterator of characters represented as an unsigned 16-bit integer
    let mut characters = if unicode {
        Either::Left(
            string_bytes
                .chunks_exact(size_of::<u16>())
                .map(LE::read_u16),
        )
    } else {
        Either::Right(string_bytes.iter().copied().map(u16::from))
    };

    let mut buf = String::new();

    while let Some(mut current) = characters.next() {
        if u8::try_from(current).is_ok_and(|code| NsCode::is_code(code, nsis_version)) {
            let Some(mut next) = characters.next() else {
                break;
            };
            if current != u16::from(NsCode::Skip.get(nsis_version)) {
                let special_char = if unicode {
                    next
                } else {
                    let Some(next_next) = characters.next() else {
                        break;
                    };
                    u16::from_le_bytes([next as u8, next_next as u8])
                };
                if current == u16::from(NsCode::Shell.get(nsis_version)) {
                    Shell::resolve(&mut buf, strings_block, special_char, nsis_version);
                } else {
                    let index = if unicode {
                        next = decode_number_from_char(special_char);
                        next
                    } else {
                        decode_number_from_char(special_char)
                    };
                    if current == u16::from(NsCode::Var.get(nsis_version)) {
                        NsVar::resolve(
                            &mut buf,
                            strings_block,
                            usize::from(index),
                            entries,
                            nsis_version,
                        );
                    } else if current == u16::from(NsCode::Lang.get(nsis_version)) {
                        LangCode::resolve(&mut buf, index);
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

    Cow::Owned(buf)
}
