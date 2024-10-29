use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use zerocopy::little_endian::U32;

#[expect(dead_code)]
#[derive(Debug)]
pub struct StrCopy {
    pub variable: U32,
    pub string_offset: U32,
    pub max_length: U32,
    pub start_position: U32,
}

impl StrCopy {
    pub fn from_entry(entry: &Entry) -> Option<Self> {
        if entry.which != WhichEntry::StrCpy {
            return None;
        }
        Some(Self {
            variable: entry.offsets[0],
            string_offset: entry.offsets[1],
            max_length: entry.offsets[2],
            start_position: entry.offsets[3],
        })
    }
}
