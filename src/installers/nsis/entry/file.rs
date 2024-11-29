use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use chrono::{DateTime, NaiveDate, Utc};
use std::borrow::Cow;
use std::ops::{BitOr, Shl};
use std::time::Duration;
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

/// Number of 100-nanosecond intervals per second
#[expect(clippy::cast_possible_truncation)]
const FILETIME_INTERVALS_PER_SEC: u64 = (Duration::from_secs(1).as_nanos() / 100) as u64;

/// Duration between 1601-01-01 and 1970-01-01 in seconds
const UNIX_EPOCH_DIFF_SECS: u64 = NaiveDate::from_ymd_opt(1970, 1, 1)
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(1601, 1, 1).unwrap())
    .num_seconds()
    .unsigned_abs();

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
