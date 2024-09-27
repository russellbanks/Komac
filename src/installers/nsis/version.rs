use crate::installers::nsis::strings::code::NsCode;
use byteorder::{ByteOrder, LE};
use derive_more::Display;
use itertools::Either;
use memchr::memmem;

#[derive(Debug, Display, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[display("{_0}.{_1}{_2}")]
pub struct NsisVersion(pub u8, pub u8, pub u8);

impl NsisVersion {
    pub const _3: Self = Self(3, 0, 0);

    pub const _2: Self = Self(2, 0, 0);

    pub const fn is_v3(self) -> bool {
        self.0 >= 3
    }

    pub fn detect(strings_block: &[u8], unicode: bool) -> Self {
        let mut nsis3_count = 0;
        let mut nsis2_count = 0;

        let char_size = if unicode {
            size_of::<u16>()
        } else {
            size_of::<u8>()
        };

        let null_indexes = if unicode {
            Either::Left(memmem::find_iter(strings_block, b"\0\0"))
        } else {
            Either::Right(memchr::memchr_iter(0, strings_block))
        };

        let mut pos = char_size;
        for index in null_indexes {
            if index == 0 {
                // Null byte(s) at the start of the string block
                continue;
            }

            let code = strings_block
                .get(pos..index)
                .filter(|string| string.len() >= char_size)
                .and_then(|string| {
                    if unicode {
                        u8::try_from(LE::read_u16(string)).ok()
                    } else {
                        string.first().copied()
                    }
                });

            if let Some(code) = code {
                if NsCode::is_code(code, Self::_3) {
                    nsis3_count += 1;
                } else if NsCode::is_code(code, Self::_2) {
                    nsis2_count += 1;
                }
            }

            pos = index + char_size;
        }

        if nsis3_count > nsis2_count {
            Self::_3
        } else {
            Self::_2
        }
    }
}
