use crate::installers::nsis::NsisError;
use crate::installers::nsis::header::Header;
use crate::installers::nsis::header::block::{BlockHeaders, BlockType};
use crate::installers::nsis::language::table::LanguageTable;
use crate::installers::nsis::registry::Registry;
use crate::installers::nsis::strings::code::NsCode;
use crate::installers::nsis::strings::shell::Shell;
use crate::installers::nsis::strings::var::NsVar;
use crate::installers::nsis::version::NsisVersion;
use byteorder::{ByteOrder, LE};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use itertools::Either;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::debug;
use yara_x::mods::PE;

pub struct NsisState<'data> {
    pub str_block: &'data [u8],
    pub language_table: &'data LanguageTable,
    pub stack: Vec<Cow<'data, str>>,
    pub variables: HashMap<usize, Cow<'data, str>>,
    pub registry: Registry<'data>,
    pub version: NsisVersion,
}

impl<'data> NsisState<'data> {
    pub fn new(
        pe: &PE,
        data: &'data [u8],
        header: &Header,
        blocks: &BlockHeaders,
    ) -> Result<Self, NsisError> {
        let mut state = Self {
            str_block: BlockType::Strings.get(data, blocks),
            language_table: LanguageTable::get_main(data, header, blocks)?,
            stack: Vec::new(),
            variables: HashMap::new(),
            registry: Registry::new(),
            version: NsisVersion::default(),
        };

        state.version = NsisVersion::from_manifest(data, pe)
            .or_else(|| NsisVersion::from_branding_text(&state))
            .unwrap_or_else(|| NsisVersion::detect(state.str_block));

        debug!(version = %state.version);

        Ok(state)
    }

    #[expect(clippy::cast_possible_truncation)] // Truncating u16 `as u8` is intentional
    pub fn get_string(&self, relative_offset: i32) -> Cow<'data, str> {
        // The strings block starts with a UTF-16 null byte if it is Unicode
        let unicode = &self.str_block[..size_of::<u16>()] == b"\0\0";

        // Double the offset if the string is Unicode as each character will be 2 bytes
        let offset = if relative_offset.is_negative() {
            // A negative offset means it's a language table
            self.language_table.string_offsets[(relative_offset + 1).unsigned_abs() as usize]
                .get()
                .unsigned_abs() as usize
        } else {
            relative_offset.unsigned_abs() as usize
        } * (usize::from(unicode) + 1);

        // Get the index of the null byte at the end of the string
        let string_end_index = if unicode {
            self.str_block[offset..]
                .chunks_exact(size_of::<u16>())
                .position(|chunk| chunk == b"\0\0")
                .map(|index| index * size_of::<u16>())
        } else {
            memchr::memchr(0, &self.str_block[offset..])
        }
        .unwrap_or(self.str_block.len());

        let string_bytes = &self.str_block[offset..offset + string_end_index];

        // Check whether the string contains any special characters that need to be decoded
        let contains_code = if unicode {
            string_bytes
                .chunks_exact(size_of::<u16>())
                .map(LE::read_u16)
                .any(|char| {
                    u8::try_from(char).is_ok_and(|code| NsCode::is_code(code, self.version))
                })
        } else {
            string_bytes
                .iter()
                .any(|&char| NsCode::is_code(char, self.version))
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
            if u8::try_from(current).is_ok_and(|code| NsCode::is_code(code, self.version)) {
                let Some(next) = characters.next() else {
                    break;
                };
                if current != u16::from(NsCode::Skip.get(self.version)) {
                    let special_char = if unicode {
                        next
                    } else {
                        let Some(next_next) = characters.next() else {
                            break;
                        };
                        u16::from_le_bytes([next as u8, next_next as u8])
                    };
                    if current == u16::from(NsCode::Shell.get(self.version)) {
                        Shell::resolve(&mut buf, self, special_char);
                    } else {
                        let index = usize::from(decode_number_from_char(special_char));
                        if current == u16::from(NsCode::Var.get(self.version)) {
                            NsVar::resolve(&mut buf, index, &self.variables, self.version);
                        } else if current == u16::from(NsCode::Lang.get(self.version)) {
                            buf.push_str(
                                &self.get_string(self.language_table.string_offsets[index].get()),
                            );
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
}

const fn decode_number_from_char(mut char: u16) -> u16 {
    const ASCII_MASK: u16 = u16::from_le_bytes([u8::MAX >> 1; size_of::<u16>()]);

    // Convert each byte into ASCII value range (0x00 - 0x7F)
    char &= ASCII_MASK;
    let le_bytes = char.to_le_bytes();
    le_bytes[0] as u16 | ((le_bytes[1] as u16) << 7)
}
