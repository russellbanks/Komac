use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use std::borrow::Cow;
use zerocopy::{try_transmute, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, TryFromBytes)]
#[repr(u32)]
pub enum RegRoot {
    ShellContext = 0u32.to_le(),
    HKeyClassesRoot = 0x8000_0000u32.to_le(),
    HKeyCurrentUser = 0x8000_0001u32.to_le(),
    HKeyLocalMachine = 0x8000_0002u32.to_le(),
    HKeyUsers = 0x8000_0003u32.to_le(),
    HKeyPerformanceData = 0x8000_0004u32.to_le(),
    HKeyCurrentConfig = 0x8000_0005u32.to_le(),
    HKeyDynamicData = 0x8000_0006u32.to_le(),
    HKeyPerformanceText = 0x8000_0050u32.to_le(),
    HKeyPerformanceNLSText = 0x8000_0060u32.to_le(),
}

#[derive(Debug)]
pub enum RegActionType {
    ExpandStr,
    Str,
    MultiStr,
    Bin,
    Dword,
}

impl RegActionType {
    pub const fn from_entry(entry: &Entry) -> Option<Self> {
        match (entry.offsets[4].get(), entry.offsets[5].get()) {
            (1, 2) | (2, _) => Some(Self::ExpandStr),
            (1, _) => Some(Self::Str),
            (3, 7) => Some(Self::MultiStr),
            (3, _) => Some(Self::Bin),
            (4, _) => Some(Self::Dword),
            _ => None,
        }
    }
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct WriteReg<'str_block> {
    pub r#type: RegActionType,
    pub root: RegRoot,
    pub key_name: Cow<'str_block, str>,
    pub value_name: Cow<'str_block, str>,
    pub value: Cow<'str_block, str>,
}

impl<'str_block> WriteReg<'str_block> {
    pub fn from_entry(
        entry: &Entry,
        strings_block: &'str_block [u8],
        nsis_version: NsisVersion,
    ) -> Option<Self> {
        if entry.which != WhichEntry::WriteReg {
            return None;
        }
        Some(Self {
            r#type: RegActionType::from_entry(entry)?,
            root: try_transmute!(entry.offsets[0]).ok()?,
            key_name: nsis_string(strings_block, entry.offsets[1].get(), nsis_version),
            value_name: nsis_string(strings_block, entry.offsets[2].get(), nsis_version),
            value: nsis_string(strings_block, entry.offsets[3].get(), nsis_version),
        })
    }
}
