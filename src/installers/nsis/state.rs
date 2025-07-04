use std::borrow::Cow;

use encoding_rs::{UTF_16LE, WINDOWS_1252};
use itertools::Either;
use tracing::debug;
use yara_x::mods::PE;
use zerocopy::{FromBytes, LE, TryFromBytes, U16, little_endian::I32};

use super::{
    Variables,
    entry::{Entry, EntryError},
};
use crate::installers::nsis::{
    NsisError,
    file_system::FileSystem,
    header::{
        Header,
        block::{BlockHeaders, BlockType},
    },
    language::table::LanguageTable,
    registry::Registry,
    strings::{code::NsCode, shell::Shell, var::NsVar},
    version::NsisVersion,
};

pub struct NsisState<'data> {
    str_block: &'data [u8],
    entries: &'data [Entry],
    pub language_table: &'data LanguageTable,
    pub stack: Vec<Cow<'data, str>>,
    pub variables: Variables<'data>,
    pub registry: Registry<'data>,
    pub file_system: FileSystem,
    version: NsisVersion,
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
            entries: <[Entry]>::try_ref_from_bytes(BlockType::Entries.get(data, blocks))
                .map_err(|error| NsisError::ZeroCopy(error.to_string()))?,
            language_table: LanguageTable::get_main(data, header, blocks)?,
            stack: Vec::new(),
            variables: Variables::new(),
            registry: Registry::new(),
            file_system: FileSystem::new(),
            version: NsisVersion::default(),
        };

        if header.install_directory_ptr != I32::ZERO {
            let install_dir = state.get_string(header.install_directory_ptr.get());
            debug!(%install_dir);
            state.variables.insert_install_dir(install_dir);
        }

        state.version = NsisVersion::from_manifest(data, pe)
            .or_else(|| NsisVersion::from_branding_text(&state))
            .unwrap_or_else(|| NsisVersion::detect(state.str_block));

        debug!(version = %state.version);

        Ok(state)
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Truncating u16 as u8 is intentional"
    )]
    pub fn get_string(&self, relative_offset: i32) -> Cow<'data, str> {
        // The strings block starts with a UTF-16 null byte if it is Unicode
        let unicode = &self.str_block[..size_of::<u16>()] == b"\0\0";

        // Double the offset if the string is Unicode as each character will be 2 bytes
        let offset = if relative_offset.is_negative() {
            // A negative offset means it's a language string from the language table
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

        let string_chars = if unicode {
            assert_eq!(string_bytes.len() % size_of::<u16>(), 0);
            Either::Left(<[U16<LE>]>::ref_from_bytes(string_bytes).unwrap())
        } else {
            Either::Right(string_bytes)
        };

        // Check whether the string contains any special characters that need to be decoded
        let contains_code = match string_chars {
            Either::Left(chars) => chars
                .iter()
                .any(|char| NsCode::is_code(char.get(), self.version)),
            Either::Right(bytes) => bytes
                .iter()
                .any(|&char| NsCode::is_code(char, self.version)),
        };

        // If the string doesn't have any special characters, we can just decode it normally
        if !contains_code {
            let encoding = if unicode { UTF_16LE } else { WINDOWS_1252 };
            return encoding.decode_without_bom_handling(string_bytes).0;
        }

        // Create an iterator of characters represented as an unsigned 16-bit integer
        let mut characters = string_chars
            .map_left(|chars| chars.iter().copied().map(U16::get))
            .map_right(|bytes| bytes.iter().copied().map(u16::from));

        let mut buf = String::new();

        while let Some(mut current) = characters.next() {
            if let Some(code) = NsCode::try_new_with_version(current, self.version) {
                let Some(next) = characters.next() else {
                    break;
                };
                if !code.is_skip() {
                    let special_char = if unicode {
                        next
                    } else {
                        let Some(next_next) = characters.next() else {
                            break;
                        };
                        u16::from_le_bytes([next as u8, next_next as u8])
                    };
                    if code.is_shell() {
                        Shell::resolve(&mut buf, self, special_char);
                    } else {
                        let index = usize::from(decode_number_from_char(special_char));
                        if code.is_var() {
                            NsVar::resolve(&mut buf, index, &self.variables, self.version);
                        } else if code.is_lang() {
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

    pub fn get_int(&self, relative_offset: i32) -> i32 {
        const HEX_START: &str = "0x";

        let mut value = &*self.get_string(relative_offset);

        let radix = if value
            .get(..HEX_START.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(HEX_START))
        {
            value = &value[HEX_START.len()..];
            16
        } else {
            10
        };

        i32::from_str_radix(value, radix).unwrap_or_default()
    }

    pub fn execute_code_segment(&mut self, mut position: i32) -> Result<i32, EntryError> {
        // Create a watchdog counter to detect infinite loops
        let mut watchdog_counter = 0;

        // Set an infinite loop threshold to the number of total entries, as we're only simulating
        // execution of a segment, not all code
        let infinite_loop_threshold = self.entries.len();

        while let Ok(index) = usize::try_from(position) {
            let entry = self.entries[index];
            let address = entry.execute(self)?;

            if entry == Entry::Return {
                return Ok(0);
            }

            if watchdog_counter >= infinite_loop_threshold {
                return Err(EntryError::InfiniteLoop);
            }

            let resolved_address = self.resolve_address(address);
            if resolved_address == 0 {
                position += 1;
            } else {
                position = resolved_address - 1; // -1 because addresses are encoded as +1
            }

            watchdog_counter += 1;
        }

        Ok(0)
    }

    pub fn resolve_address(&self, address: i32) -> i32 {
        if address.is_negative() {
            self.variables
                .get(&((address.unsigned_abs() - 1) as usize))
                .and_then(|address| address.parse().ok())
                .unwrap()
        } else {
            address
        }
    }
}

const fn decode_number_from_char(mut char: u16) -> u16 {
    const ASCII_MASK: u16 = u16::from_le_bytes([u8::MAX >> 1; size_of::<u16>()]);

    // Convert each byte into ASCII value range (0x00 - 0x7F)
    char &= ASCII_MASK;
    let le_bytes = char.to_le_bytes();
    le_bytes[0] as u16 | ((le_bytes[1] as u16) << 7)
}
