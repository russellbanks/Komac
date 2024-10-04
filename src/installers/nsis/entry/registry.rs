use crate::installers::nsis::entry::which::WhichEntry;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use std::borrow::Cow;
use strum::FromRepr;

#[derive(Debug, FromRepr)]
#[repr(u32)]
pub enum RegRoot {
    ShellContext = 0,
    HKeyClassesRoot = 0x8000_0000,
    HKeyCurrentUser = 0x8000_0001,
    HKeyLocalMachine = 0x8000_0002,
    HKeyUsers = 0x8000_0003,
    HKeyPerformanceData = 0x8000_0004,
    HKeyCurrentConfig = 0x8000_0005,
    HKeyDynamicData = 0x8000_0006,
    HKeyPerformanceText = 0x8000_0050,
    HKeyPerformanceNLSText = 0x8000_0060,
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
            root: RegRoot::from_repr(entry.offsets[0].get())?,
            key_name: nsis_string(strings_block, entry.offsets[1].get(), nsis_version),
            value_name: nsis_string(strings_block, entry.offsets[2].get(), nsis_version),
            value: nsis_string(strings_block, entry.offsets[3].get(), nsis_version),
        })
    }
}
