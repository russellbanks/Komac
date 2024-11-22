use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[expect(dead_code)]
#[derive(Debug, FromBytes, KnownLayout, Immutable)]
pub struct StrCopy {
    pub variable: U32,
    pub string_offset: U32,
    pub max_length: U32,
    pub start_position: U32,
}

impl StrCopy {
    pub fn from_entry(entry: &Entry) -> Option<&Self> {
        if entry.which != WhichEntry::StrCpy {
            return None;
        }
        Self::ref_from_prefix(entry.offsets.as_bytes())
            .map(|(str_copy, _)| str_copy)
            .ok()
    }
}
