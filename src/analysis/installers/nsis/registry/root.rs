use std::{fmt, fmt::Formatter};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq, FromBytes, KnownLayout, Immutable)]
pub struct RegRoot(U32<LE>);

impl RegRoot {
    const REG_ROOT_VIEW_32: u32 = 0x4000_0000;
    const REG_ROOT_VIEW_64: u32 = 0x2000_0000;
    const REG_ROOT_VIEW_ANY: u32 = Self::REG_ROOT_VIEW_32 | Self::REG_ROOT_VIEW_64;

    pub const SHELL_CONTEXT: u32 = 0;

    // Native Windows predefined roots
    pub const HKEY_CLASSES_ROOT: u32 = 0x8000_0000;
    pub const HKEY_CURRENT_USER: u32 = 0x8000_0001;
    pub const HKEY_LOCAL_MACHINE: u32 = 0x8000_0002;
    pub const HKEY_USERS: u32 = 0x8000_0003;
    pub const HKEY_PERFORMANCE_DATA: u32 = 0x8000_0004;
    pub const HKEY_CURRENT_CONFIG: u32 = 0x8000_0005;
    pub const HKEY_DYNAMIC_DATA: u32 = 0x8000_0006;
    pub const HKEY_PERFORMANCE_TEXT: u32 = 0x8000_0050;
    pub const HKEY_PERFORMANCE_NLSTEXT: u32 = 0x8000_0060;

    // Force 32-bit view
    pub const SHELL_CONTEXT32: u32 = Self::SHELL_CONTEXT | Self::REG_ROOT_VIEW_32;
    pub const HKEY_CLASSES_ROOT32: u32 = Self::HKEY_CLASSES_ROOT | Self::REG_ROOT_VIEW_32;
    pub const HKEY_CURRENT_USER32: u32 = Self::HKEY_CURRENT_USER | Self::REG_ROOT_VIEW_32;
    pub const HKEY_LOCAL_MACHINE32: u32 = Self::HKEY_LOCAL_MACHINE | Self::REG_ROOT_VIEW_32;

    // Force 64-bit view
    pub const SHELL_CONTEXT64: u32 = Self::SHELL_CONTEXT | Self::REG_ROOT_VIEW_64;
    pub const HKEY_CLASSES_ROOT64: u32 = Self::HKEY_CLASSES_ROOT | Self::REG_ROOT_VIEW_64;
    pub const HKEY_CURRENT_USER64: u32 = Self::HKEY_CURRENT_USER | Self::REG_ROOT_VIEW_64;
    pub const HKEY_LOCAL_MACHINE64: u32 = Self::HKEY_LOCAL_MACHINE | Self::REG_ROOT_VIEW_64;

    // Either view allowed
    pub const SHELL_CONTEXT_ANY: u32 = Self::SHELL_CONTEXT | Self::REG_ROOT_VIEW_ANY;
    pub const HKEY_CLASSES_ROOT_ANY: u32 = Self::HKEY_CLASSES_ROOT | Self::REG_ROOT_VIEW_ANY;
    pub const HKEY_CURRENT_USER_ANY: u32 = Self::HKEY_CURRENT_USER | Self::REG_ROOT_VIEW_ANY;
    pub const HKEY_LOCAL_MACHINE_ANY: u32 = Self::HKEY_LOCAL_MACHINE | Self::REG_ROOT_VIEW_ANY;
}

impl fmt::Display for RegRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0.get() {
            Self::SHELL_CONTEXT => f.write_str("SHELL_CONTEXT"),
            Self::HKEY_CLASSES_ROOT => f.write_str("HKEY_CLASSES_ROOT"),
            Self::HKEY_CURRENT_USER => f.write_str("HKEY_CURRENT_USER"),
            Self::HKEY_LOCAL_MACHINE => f.write_str("HKEY_LOCAL_MACHINE"),
            Self::HKEY_USERS => f.write_str("HKEY_USERS"),
            Self::HKEY_PERFORMANCE_DATA => f.write_str("HKEY_PERFORMANCE_DATA"),
            Self::HKEY_CURRENT_CONFIG => f.write_str("HKEY_CURRENT_CONFIG"),
            Self::HKEY_DYNAMIC_DATA => f.write_str("HKEY_DYNAMIC_DATA"),
            Self::HKEY_PERFORMANCE_TEXT => f.write_str("HKEY_PERFORMANCE"),
            Self::HKEY_PERFORMANCE_NLSTEXT => f.write_str("HKEY_PERFORMANCE_NLSTEXT"),
            Self::SHELL_CONTEXT64 => f.write_str("SHELL_CONTEXT_64"),
            Self::HKEY_CLASSES_ROOT64 => f.write_str("HKEY_CLASSES_ROOT_64"),
            Self::HKEY_CURRENT_USER64 => f.write_str("HKEY_CURRENT_USER_64"),
            Self::HKEY_LOCAL_MACHINE64 => f.write_str("HKEY_LOCAL_MACHINE_64"),
            Self::SHELL_CONTEXT32 => f.write_str("SHELL_CONTEXT_32"),
            Self::HKEY_CLASSES_ROOT32 => f.write_str("HKEY_CLASSES_ROOT_32"),
            Self::HKEY_CURRENT_USER32 => f.write_str("HKEY_CURRENT_USER_32"),
            Self::HKEY_LOCAL_MACHINE32 => f.write_str("HKEY_LOCAL_MACHINE_32"),
            Self::SHELL_CONTEXT_ANY => f.write_str("SHELL_CONTEXT_ANY"),
            Self::HKEY_CLASSES_ROOT_ANY => f.write_str("HKEY_CLASSES_ROOT_ANY"),
            Self::HKEY_CURRENT_USER_ANY => f.write_str("HKEY_CURRENT_USER_ANY"),
            Self::HKEY_LOCAL_MACHINE_ANY => f.write_str("HKEY_LOCAL_MACHINE_ANY"),
            root => write!(f, "{root:#X}"),
        }
    }
}
