use serde::Deserialize;
use uuid::{Uuid, fmt::Braced};

use super::{arp::Arp, bool_from_yes_no};

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L217
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Registration {
    #[serde(rename = "@Code", alias = "@Id")]
    code: Uuid,
    #[serde(rename = "@ExecutableName")]
    pub executable_name: String,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    per_machine: bool,
    scope: Option<WixBundleScope>,
    #[serde(rename = "@Tag")]
    pub tag: String,
    #[serde(rename = "@Version")]
    pub version: String,
    #[serde(rename = "@ProviderKey")]
    pub provider_key: String,
    pub arp: Arp,
}

impl Registration {
    #[inline]
    pub fn code(&self) -> &Braced {
        self.code.as_braced()
    }

    pub fn scope(&self) -> WixBundleScope {
        self.scope
            .unwrap_or_else(|| WixBundleScope::from(self.per_machine))
    }
}

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/api/wix/WixToolset.Data/Symbols/WixBundleSymbol.cs#L93>
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum WixBundleScope {
    /// The package is a per-machine installation and requires elevated privileges to install.
    ///
    /// Sets the `ALLUSERS` property to 1.
    PerMachine,

    /// The package is dual-purpose that can install per-machine or per-user and defaults to
    /// installing per-machine.
    ///
    /// Sets the `ALLUSERS` property to 2.
    PerMachineOrUser,

    /// The package is dual-purpose that can install per-user or per-machine and defaults to
    /// installing per-user.
    ///
    /// Sets the `ALLUSERS` property to 2 and `MSIINSTALLPERUSER` property to 1.
    PerUserOrMachine,

    /// The package is a per-user installation and does not require elevated privileges to install.
    ///
    /// Sets the package’s InstallPrivileges attribute to “limited.”
    PerUser,
}

impl From<bool> for WixBundleScope {
    fn from(per_machine: bool) -> Self {
        if per_machine {
            Self::PerMachine
        } else {
            Self::PerUser
        }
    }
}
