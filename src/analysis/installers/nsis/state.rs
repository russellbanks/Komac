use itertools::Either;
use tracing::{debug, debug_span};
use zerocopy::{FromBytes, I32, LE, U16};

use super::{
    Variables,
    entry::{Entry, EntryError, ExecFlags, system::MockCaller},
};
use crate::analysis::installers::nsis::{
    NsisError,
    file_system::FileSystem,
    header::Header,
    language::table::LanguageTable,
    registry::Registry,
    strings::{NsisChar, NsisCow, NsisStr, NsisString, code::NsCode, shell::Shell, var::NsVar},
    version::NsisVersion,
};

pub struct NsisState<'data> {
    pub str_block: &'data [u8],
    entries: &'data [Entry],
    pub language_table: &'data LanguageTable,
    pub stack: Vec<NsisCow<'data>>,
    pub variables: Variables<'data>,
    pub registry: Registry,
    pub file_system: FileSystem,
    pub exec_flags: ExecFlags,
    pub last_used_exec_flags: ExecFlags,
    pub status_up_hack: I32<LE>,
    pub mock_caller: MockCaller,
    version: NsisVersion,
}

impl<'data> NsisState<'data> {
    pub fn new(
        data: &'data [u8],
        header: &Header,
        manifest: Option<&str>,
    ) -> Result<Self, NsisError> {
        let mut state = Self {
            str_block: header.blocks().strings_block(data),
            entries: header.blocks().entries(data)?,
            language_table: LanguageTable::primary_language(data, header.blocks())?,
            stack: Vec::new(),
            variables: Variables::new(),
            registry: Registry::new(),
            file_system: FileSystem::new(),
            exec_flags: ExecFlags::new(),
            last_used_exec_flags: ExecFlags::new(),
            status_up_hack: I32::ZERO,
            mock_caller: MockCaller::new(),
            version: NsisVersion::default(),
        };

        state.version = manifest
            .and_then(NsisVersion::from_manifest)
            .or_else(|| NsisVersion::from_branding_text(&state))
            .unwrap_or_else(|| NsisVersion::detect(state.str_block));

        debug!(version = %state.version);

        // The install directory must be read after the version is determined; `get_string` depends on it
        if let Some(install_directory_ptr) = header.install_directory() {
            let install_dir = state.get_string(install_directory_ptr);
            debug!(%install_dir);
            state.variables.insert_install_dir(install_dir);
        }

        Ok(state)
    }

    /// Returns the version of the NSIS executable.
    #[must_use]
    #[inline]
    pub const fn version(&self) -> NsisVersion {
        self.version
    }

    /// Returns `true` if the NSIS executable is Unicode, `false` otherwise.
    #[must_use]
    #[inline]
    pub fn is_unicode(&self) -> bool {
        // The strings block starts with a UTF-16 null byte if it is Unicode
        self.str_block.starts_with(b"\0\0")
    }

    /// Encodes a UTF-8 string in this executable's native string encoding.
    #[must_use]
    pub fn encode(&self, string: &str) -> NsisString {
        NsisString::from_str(string, self.is_unicode())
    }

    #[inline]
    pub const fn is_park(&self) -> bool {
        self.version.is_park()
    }

    /// Returns `true` if the NSIS executable is portable.
    ///
    /// <https://www.electron.build/nsis.html#portable>
    pub fn is_portable(&self) -> bool {
        const PORTABLE_EXECUTABLE_DIR: &str = "PORTABLE_EXECUTABLE_DIR";
        const PORTABLE_EXECUTABLE_FILE: &str = "PORTABLE_EXECUTABLE_FILE";
        const PORTABLE_EXECUTABLE_APP_FILENAME: &str = "PORTABLE_EXECUTABLE_APP_FILENAME";
        const PORTABLE_ENVIRONMENT_VARIABLES: [&str; 3] = [
            PORTABLE_EXECUTABLE_DIR,
            PORTABLE_EXECUTABLE_FILE,
            PORTABLE_EXECUTABLE_APP_FILENAME,
        ];

        self.mock_caller
            .kernel32()
            .environment_variables()
            .keys()
            .any(|key| PORTABLE_ENVIRONMENT_VARIABLES.contains(&&**key))
    }

    /// Resolves the string at `relative_offset` in the strings block, substituting any variables,
    /// shell folders, and language strings it references. A negative offset refers to a string in
    /// the language table. The string is borrowed from the strings block when it needs no
    /// substitution, and is `$ERROR` if the offset is invalid.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Truncating u16 as u8 is intentional"
    )]
    pub fn get_string(&self, relative_offset: i32) -> NsisCow<'data> {
        // The strings block starts with a UTF-16 null byte if it is Unicode
        let unicode = self.is_unicode();

        // Double the offset if the string is Unicode as each character will be 2 bytes
        let offset = if relative_offset.is_negative() {
            // A negative offset means it's a language string from the language table
            let Some(offset) = self
                .language_table
                .string_offset((relative_offset + 1).unsigned_abs() as usize)
            else {
                return NsisCow::Borrowed(NsisStr::error());
            };

            offset.unsigned_abs() as usize
        } else {
            relative_offset.unsigned_abs() as usize
        } * (usize::from(unicode) + 1);

        let Some(offset_slice) = self.str_block.get(offset..) else {
            return NsisCow::Borrowed(NsisStr::error());
        };

        // Get the index of the null byte at the end of the string
        let string_end_index = if unicode {
            let (chunks, _remainder) = offset_slice.as_chunks();
            chunks
                .iter()
                .position(|chunk| chunk == b"\0\0")
                .map(|index| index * size_of::<u16>())
        } else {
            memchr::memchr(0, offset_slice)
        }
        .unwrap_or(offset_slice.len());

        let string_bytes = &self.str_block[offset..offset + string_end_index];

        let string_chars = if unicode {
            assert_eq!(string_bytes.len() % size_of::<u16>(), 0);
            Either::Left(<[U16<LE>]>::ref_from_bytes(string_bytes).unwrap())
        } else {
            Either::Right(string_bytes)
        };

        if self.is_park() {
            todo!("NSIS Park support");
        }

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
            return NsisCow::Borrowed(NsisStr::from_bytes(string_bytes, unicode));
        }

        // Create an iterator of characters represented as an unsigned 16-bit integer
        let mut characters = string_chars
            .map_left(|chars| chars.iter().copied().map(U16::get))
            .map_right(|bytes| bytes.iter().copied().map(u16::from));

        let mut buf = NsisString::new(unicode);

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
                        } else if code.is_lang()
                            && let Some(offset) = self.language_table.string_offset(index)
                        {
                            buf.push_nsis_str(self.get_string(offset).as_nsis_str());
                        }
                    }
                    continue;
                }
                current = next;
            }
            buf.push(if unicode {
                NsisChar::Wide(U16::new(current))
            } else {
                NsisChar::Ansi(current as u8)
            });
        }

        NsisCow::Owned(buf)
    }

    /// Resolves the string at `relative_offset` and parses it as a decimal integer, or as
    /// hexadecimal if it starts with `0x`. Returns `0` if the string is not a valid integer.
    pub fn get_int(&self, relative_offset: i32) -> i32 {
        const HEX_START: &str = "0x";

        let string = self.get_string(relative_offset);
        let mut value = string.as_nsis_str();

        let radix = if value.starts_with_ignore_ascii_case(HEX_START) {
            value = value.slice(HEX_START.len()..value.len());
            16
        } else {
            10
        };

        value.parse_int(radix).unwrap_or_default()
    }

    /// Execute a NSIS code section.
    ///
    /// <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L79>
    pub fn execute_code_segment(&mut self, mut position: i32) -> Result<Entry, EntryError> {
        // Reset file system directory to root
        self.file_system.set_root();

        // Create a watchdog counter to detect infinite loops
        let mut watchdog_counter = 0;

        // Set an infinite loop threshold to the number of total entries, as we're only simulating
        // execution of a segment, not all code
        let infinite_loop_threshold = self.entries.len();

        while let Ok(index) = usize::try_from(position) {
            let span = debug_span!("", "{}", index + 1);
            let _enter = span.enter();

            let entry = self.entries[index];
            let address = entry.execute(self)?;

            if matches!(entry, Entry::Return | Entry::Quit) {
                return Ok(entry);
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

        Ok(Entry::Return)
    }

    /// Resolves a jump address. A negative address refers to a variable that holds the address as a
    /// decimal integer.
    pub fn resolve_address(&self, address: i32) -> i32 {
        if address.is_negative() {
            self.variables
                .get(&((address.unsigned_abs() - 1) as usize))
                .and_then(|address| address.parse_int(10))
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
