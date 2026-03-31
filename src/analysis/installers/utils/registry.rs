use std::{fmt, fmt::Formatter};

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(
    Copy, Clone, Debug, Default, Hash, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable,
)]
#[repr(u32)]
pub enum RegRoot {
    #[default]
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
    ShellContext64 = 0x2000_0000u32.to_le(),
    HKeyClassesRoot64 = 0xA000_0000u32.to_le(),
    HKeyCurrentUser64 = 0xA000_0001u32.to_le(),
    HKeyLocalMachine64 = 0xA000_0002u32.to_le(),
    HKeyUsers64 = 0xA000_0003u32.to_le(),
    HKeyPerformanceData64 = 0xA000_0004u32.to_le(),
    HKeyCurrentConfig64 = 0xA000_0005u32.to_le(),
    HKeyDynamicData64 = 0xA000_0006u32.to_le(),
    HKeyPerformanceText64 = 0xA000_0050u32.to_le(),
    HKeyPerformanceNLSText64 = 0xA000_0060u32.to_le(),
}

impl fmt::Display for RegRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShellContext => f.write_str("SHELL_CONTEXT"),
            Self::HKeyClassesRoot => f.write_str("HKEY_CLASSES_ROOT"),
            Self::HKeyCurrentUser => f.write_str("HKEY_CURRENT_USER"),
            Self::HKeyLocalMachine => f.write_str("HKEY_LOCAL_MACHINE"),
            Self::HKeyUsers => f.write_str("HKEY_USERS"),
            Self::HKeyPerformanceData => f.write_str("HKEY_PERFORMANCE_DATA"),
            Self::HKeyCurrentConfig => f.write_str("HKEY_CURRENT_CONFIG"),
            Self::HKeyDynamicData => f.write_str("HKEY_DYNAMIC_DATA"),
            Self::HKeyPerformanceText => f.write_str("HKEY_PERFORMANCE"),
            Self::HKeyPerformanceNLSText => f.write_str("HKEY_PERFORMANCE_NLSTEXT"),
            Self::ShellContext64 => f.write_str("SHELL_CONTEXT (64-bit)"),
            Self::HKeyClassesRoot64 => f.write_str("HKEY_CLASSES_ROOT (64-bit)"),
            Self::HKeyCurrentUser64 => f.write_str("HKEY_CURRENT_USER (64-bit)"),
            Self::HKeyLocalMachine64 => f.write_str("HKEY_LOCAL_MACHINE (64-bit)"),
            Self::HKeyUsers64 => f.write_str("HKEY_USERS (64-bit)"),
            Self::HKeyPerformanceData64 => f.write_str("HKEY_PERFORMANCE_DATA (64-bit)"),
            Self::HKeyCurrentConfig64 => f.write_str("HKEY_CURRENT_CONFIG (64-bit)"),
            Self::HKeyDynamicData64 => f.write_str("HKEY_DYNAMIC_DATA (64-bit)"),
            Self::HKeyPerformanceText64 => f.write_str("HKEY_PERFORMANCE (64-bit)"),
            Self::HKeyPerformanceNLSText64 => f.write_str("HKEY_PERFORMANCE_NLSTEXT (64-bit)"),
        }
    }
}
