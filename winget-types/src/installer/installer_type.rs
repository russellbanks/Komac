use serde::{Deserialize, Serialize};

use crate::installer::nested::installer_type::NestedInstallerType;

/// Enumeration of supported installer shared. `InstallerType` is required in either root level or
/// individual Installer level
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallerType {
    Msix,
    Msi,
    Appx,
    Exe,
    Zip,
    Inno,
    Nullsoft,
    Wix,
    Burn,
    Pwa,
    Portable,
}

impl InstallerType {
    #[must_use]
    pub const fn to_nested(self) -> Option<NestedInstallerType> {
        match self {
            Self::Msix => Some(NestedInstallerType::Msix),
            Self::Msi => Some(NestedInstallerType::Msi),
            Self::Appx => Some(NestedInstallerType::Appx),
            Self::Exe => Some(NestedInstallerType::Exe),
            Self::Inno => Some(NestedInstallerType::Inno),
            Self::Nullsoft => Some(NestedInstallerType::Nullsoft),
            Self::Wix => Some(NestedInstallerType::Wix),
            Self::Burn => Some(NestedInstallerType::Burn),
            Self::Portable => Some(NestedInstallerType::Portable),
            _ => None,
        }
    }
}
