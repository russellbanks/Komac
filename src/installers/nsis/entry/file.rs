use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use chrono::{DateTime, Utc};
use nt_time::FileTime;
use std::borrow::Cow;
use zerocopy::{try_transmute, Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum OverwriteFlag {
    #[default]
    Force = 0u32.to_le(),
    No = 1u32.to_le(),
    Try = 2u32.to_le(),
    IfDateIsNewer = 3u32.to_le(),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct ExtractFile<'str_block> {
    overwrite_flag: OverwriteFlag,
    pub name: Cow<'str_block, str>,
    pub position: usize,
    filetime: DateTime<Utc>,
    allow_ignore: u32,
}

enum Offsets {
    OverwriteFlag,
    Filename,
    FilePosition,
    FileDateTimeLow,
    FileDateTimeHigh,
    AllowIgnore,
}

impl<'str_block> ExtractFile<'str_block> {
    pub fn from_entry(
        entry: &Entry,
        strings_block: &'str_block [u8],
        nsis_version: NsisVersion,
    ) -> Option<Self> {
        if entry.which != WhichEntry::ExtractFile {
            return None;
        }
        Some(Self {
            overwrite_flag: try_transmute!(entry.offsets[Offsets::OverwriteFlag as usize] & 0b111)
                .unwrap_or_default(),
            name: nsis_string(
                strings_block,
                entry.offsets[Offsets::Filename as usize].get(),
                &[],
                nsis_version,
            ),
            position: entry.offsets[Offsets::FilePosition as usize].get() as usize
                + size_of::<u32>(),
            filetime: {
                let high = u64::from(entry.offsets[Offsets::FileDateTimeHigh as usize].get());
                let low = u64::from(entry.offsets[Offsets::FileDateTimeLow as usize].get());

                FileTime::new(high << u32::BITS | low).into()
            },
            allow_ignore: entry.offsets[Offsets::AllowIgnore as usize].get(),
        })
    }
}
