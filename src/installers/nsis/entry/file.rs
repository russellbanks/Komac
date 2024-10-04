use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use chrono::{DateTime, Utc};
use std::borrow::Cow;
use std::ops::{BitOr, Shl};
use std::time::Duration;
use strum::FromRepr;

#[derive(Debug, Default, FromRepr)]
#[repr(u32)]
enum OverwriteFlag {
    #[default]
    Force,
    No,
    Try,
    IfDateIsNewer,
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

/// Number of 100-nanosecond intervals per second
#[expect(clippy::cast_possible_truncation)]
const FILETIME_INTERVALS_PER_SEC: u64 = (Duration::from_secs(1).as_nanos() / 100) as u64;

/// Duration between 1601-01-01 and 1970-01-01 in seconds
const UNIX_EPOCH_DIFF_SECS: u64 = 11_644_473_600;

#[expect(dead_code)]
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
            overwrite_flag: OverwriteFlag::from_repr(
                entry.offsets[Offsets::OverwriteFlag as usize].get() & 0b111,
            )
            .unwrap_or_default(),
            name: nsis_string(
                strings_block,
                entry.offsets[Offsets::Filename as usize].get(),
                nsis_version,
            ),
            position: entry.offsets[Offsets::FilePosition as usize].get() as usize
                + size_of::<u32>(),
            filetime: u64::from(entry.offsets[Offsets::FileDateTimeHigh as usize].get())
                .shl(u32::BITS)
                .bitor(u64::from(
                    entry.offsets[Offsets::FileDateTimeLow as usize].get(),
                ))
                .div_euclid(FILETIME_INTERVALS_PER_SEC)
                .checked_sub(UNIX_EPOCH_DIFF_SECS)
                .and_then(|secs| DateTime::<Utc>::from_timestamp(i64::try_from(secs).ok()?, 0))
                .unwrap_or_default(),
            allow_ignore: entry.offsets[5].get(),
        })
    }
}
