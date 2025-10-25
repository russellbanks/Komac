use std::{
    fmt,
    ops::{Index, IndexMut},
};

use strum::EnumCount;
use zerocopy::{FromBytes, FromZeros, I32, Immutable, KnownLayout, LE, TryFromBytes};

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable, EnumCount)]
#[repr(u32)]
pub enum ExecFlag {
    AutoClose = 0u32,
    ShellVarContext = 1u32.to_le(),
    Errors = 2u32.to_le(),
    Abort = 3u32.to_le(),
    Reboot = 4u32.to_le(),
    RebootCalled = 5u32.to_le(),
    CurInstType = 6u32.to_le(),
    PluginApiVersion = 7u32.to_le(),
    Silent = 8u32.to_le(),
    InstDirError = 9u32.to_le(),
    RightToLeft = 10u32.to_le(),
    ErrorLevel = 11u32.to_le(),
    RegView = 12u32.to_le(),
    DetailsPrint = 13u32.to_le(),
}

impl ExecFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AutoClose => "AutoClose",
            Self::ShellVarContext => "ShellVarContext",
            Self::Errors => "Errors",
            Self::Abort => "Abort",
            Self::Reboot => "Reboot",
            Self::RebootCalled => "RebootCalled",
            Self::CurInstType => "CurInstType",
            Self::PluginApiVersion => "PluginApiVersion",
            Self::Silent => "Silent",
            Self::InstDirError => "InstDirError",
            Self::RightToLeft => "RightToLeft",
            Self::ErrorLevel => "ErrorLevel",
            Self::RegView => "RegView",
            Self::DetailsPrint => "DetailsPrint",
        }
    }
}

impl fmt::Display for ExecFlag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/api.h#L41>
#[derive(Copy, Clone, Debug, FromBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct ExecFlags([I32<LE>; ExecFlag::COUNT]);

impl ExecFlags {
    #[inline]
    pub fn new() -> Self {
        Self::new_zeroed()
    }
}

impl Index<ExecFlag> for ExecFlags {
    type Output = I32<LE>;

    fn index(&self, r#type: ExecFlag) -> &Self::Output {
        self.0.index(r#type as usize)
    }
}

impl IndexMut<ExecFlag> for ExecFlags {
    fn index_mut(&mut self, r#type: ExecFlag) -> &mut Self::Output {
        self.0.index_mut(r#type as usize)
    }
}

impl Default for ExecFlags {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
