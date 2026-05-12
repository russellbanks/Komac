use std::{cmp::Ordering, fmt, fmt::Formatter};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Copy, Clone, Hash, PartialEq, Eq, FromBytes, KnownLayout, Immutable)]
pub struct RegRoot(U32<LE>);

impl RegRoot {
    const REG_ROOT_VIEW_32: u32 = 0x4000_0000;
    const REG_ROOT_VIEW_64: u32 = 0x2000_0000;
    const REG_ROOT_VIEW_ANY: u32 = Self::REG_ROOT_VIEW_32 | Self::REG_ROOT_VIEW_64;

    pub const SHELL_CONTEXT: Self = Self(U32::ZERO);

    // Native Windows predefined roots
    pub const HKEY_CLASSES_ROOT: Self = Self::new(0x8000_0000);
    pub const HKEY_CURRENT_USER: Self = Self::new(0x8000_0001);
    pub const HKEY_LOCAL_MACHINE: Self = Self::new(0x8000_0002);
    pub const HKEY_USERS: Self = Self::new(0x8000_0003);
    pub const HKEY_PERFORMANCE_DATA: Self = Self::new(0x8000_0004);
    pub const HKEY_CURRENT_CONFIG: Self = Self::new(0x8000_0005);
    pub const HKEY_DYNAMIC_DATA: Self = Self::new(0x8000_0006);
    pub const HKEY_PERFORMANCE_TEXT: Self = Self::new(0x8000_0050);
    pub const HKEY_PERFORMANCE_NLSTEXT: Self = Self::new(0x8000_0060);

    // Force 32-bit view
    pub const SHELL_CONTEXT32: Self = Self::new(Self::SHELL_CONTEXT.get() | Self::REG_ROOT_VIEW_32);
    pub const HKEY_CLASSES_ROOT32: Self =
        Self::new(Self::HKEY_CLASSES_ROOT.get() | Self::REG_ROOT_VIEW_32);
    pub const HKEY_CURRENT_USER32: Self =
        Self::new(Self::HKEY_CURRENT_USER.get() | Self::REG_ROOT_VIEW_32);
    pub const HKEY_LOCAL_MACHINE32: Self =
        Self::new(Self::HKEY_LOCAL_MACHINE.get() | Self::REG_ROOT_VIEW_32);

    // Force 64-bit view
    pub const SHELL_CONTEXT64: Self = Self::new(Self::SHELL_CONTEXT.get() | Self::REG_ROOT_VIEW_64);
    pub const HKEY_CLASSES_ROOT64: Self =
        Self::new(Self::HKEY_CLASSES_ROOT.get() | Self::REG_ROOT_VIEW_64);
    pub const HKEY_CURRENT_USER64: Self =
        Self::new(Self::HKEY_CURRENT_USER.get() | Self::REG_ROOT_VIEW_64);
    pub const HKEY_LOCAL_MACHINE64: Self =
        Self::new(Self::HKEY_LOCAL_MACHINE.get() | Self::REG_ROOT_VIEW_64);

    // Either view allowed
    pub const SHELL_CONTEXT_ANY: Self =
        Self::new(Self::SHELL_CONTEXT.get() | Self::REG_ROOT_VIEW_ANY);
    pub const HKEY_CLASSES_ROOT_ANY: Self =
        Self::new(Self::HKEY_CLASSES_ROOT.get() | Self::REG_ROOT_VIEW_ANY);
    pub const HKEY_CURRENT_USER_ANY: Self =
        Self::new(Self::HKEY_CURRENT_USER.get() | Self::REG_ROOT_VIEW_ANY);
    pub const HKEY_LOCAL_MACHINE_ANY: Self =
        Self::new(Self::HKEY_LOCAL_MACHINE.get() | Self::REG_ROOT_VIEW_ANY);

    #[inline]
    const fn new(value: u32) -> Self {
        Self(U32::new(value))
    }

    #[inline]
    const fn get(self) -> u32 {
        self.0.get()
    }

    /// Returns the registry root as a static string slice if it's known, or `None` otherwise.
    const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::SHELL_CONTEXT => Some("SHELL_CONTEXT"),
            Self::HKEY_CLASSES_ROOT => Some("HKEY_CLASSES_ROOT"),
            Self::HKEY_CURRENT_USER => Some("HKEY_CURRENT_USER"),
            Self::HKEY_LOCAL_MACHINE => Some("HKEY_LOCAL_MACHINE"),
            Self::HKEY_USERS => Some("HKEY_USERS"),
            Self::HKEY_PERFORMANCE_DATA => Some("HKEY_PERFORMANCE_DATA"),
            Self::HKEY_CURRENT_CONFIG => Some("HKEY_CURRENT_CONFIG"),
            Self::HKEY_DYNAMIC_DATA => Some("HKEY_DYNAMIC_DATA"),
            Self::HKEY_PERFORMANCE_TEXT => Some("HKEY_PERFORMANCE"),
            Self::HKEY_PERFORMANCE_NLSTEXT => Some("HKEY_PERFORMANCE_NLSTEXT"),
            Self::SHELL_CONTEXT64 => Some("SHELL_CONTEXT_64"),
            Self::HKEY_CLASSES_ROOT64 => Some("HKEY_CLASSES_ROOT_64"),
            Self::HKEY_CURRENT_USER64 => Some("HKEY_CURRENT_USER_64"),
            Self::HKEY_LOCAL_MACHINE64 => Some("HKEY_LOCAL_MACHINE_64"),
            Self::SHELL_CONTEXT32 => Some("SHELL_CONTEXT_32"),
            Self::HKEY_CLASSES_ROOT32 => Some("HKEY_CLASSES_ROOT_32"),
            Self::HKEY_CURRENT_USER32 => Some("HKEY_CURRENT_USER_32"),
            Self::HKEY_LOCAL_MACHINE32 => Some("HKEY_LOCAL_MACHINE_32"),
            Self::SHELL_CONTEXT_ANY => Some("SHELL_CONTEXT_ANY"),
            Self::HKEY_CLASSES_ROOT_ANY => Some("HKEY_CLASSES_ROOT_ANY"),
            Self::HKEY_CURRENT_USER_ANY => Some("HKEY_CURRENT_USER_ANY"),
            Self::HKEY_LOCAL_MACHINE_ANY => Some("HKEY_LOCAL_MACHINE_ANY"),
            _ => None,
        }
    }
}

impl fmt::Debug for RegRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Default for RegRoot {
    #[inline]
    fn default() -> Self {
        Self::SHELL_CONTEXT
    }
}

impl fmt::Display for RegRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(str) = self.as_str() {
            f.write_str(str)
        } else {
            write!(f, "{:#X}", self.0)
        }
    }
}

impl Ord for RegRoot {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.as_str(), other.as_str()) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => self.0.cmp(&other.0),
        }
    }
}

impl PartialOrd for RegRoot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
