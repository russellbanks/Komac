use serde::Deserialize;
use uuid::{Uuid, fmt::Braced};

use super::{arp::Arp, bool_from_yes_no};

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L217>
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Registration {
    #[serde(rename = "@Code", alias = "@Id")]
    code: Uuid,
    #[serde(rename = "@ExecutableName")]
    pub executable_name: String,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    per_machine: bool,
    /// Written by WiX 7+ in place of `@PerMachine`.
    #[serde(rename = "@Scope")]
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

/// <https://github.com/wixtoolset/wix/blob/v7.0.0/src/api/wix/WixToolset.Data/Symbols/WixBundleSymbol.cs#L155>
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum WixBundleScope {
    /// The package is a per-machine installation and requires elevated privileges to install.
    ///
    /// Sets the `ALLUSERS` property to 1.
    #[serde(rename = "perMachine")]
    Machine,

    /// The package is dual-purpose that can install per-machine or per-user and defaults to
    /// installing per-machine.
    ///
    /// Sets the `ALLUSERS` property to 2.
    #[serde(rename = "perMachineOrUser")]
    MachineOrUser,

    /// The package is dual-purpose that can install per-user or per-machine and defaults to
    /// installing per-user.
    ///
    /// Sets the `ALLUSERS` property to 2 and `MSIINSTALLPERUSER` property to 1.
    #[serde(rename = "perUserOrMachine")]
    UserOrMachine,

    /// The package is a per-user installation and does not require elevated privileges to install.
    ///
    /// Sets the package’s `InstallPrivileges` attribute to “limited.”
    #[serde(rename = "perUser")]
    User,
}

impl From<bool> for WixBundleScope {
    fn from(per_machine: bool) -> Self {
        if per_machine {
            Self::Machine
        } else {
            Self::User
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Registration, WixBundleScope};

    const ARP: &str = r#"<Arp DisplayName="Test" DisplayVersion="1.0.0" />"#;

    fn registration(scope_attribute: &str) -> Registration {
        quick_xml::de::from_str(&format!(
            r#"<Registration Code="{{00000000-0000-0000-0000-000000000000}}" ExecutableName="setup.exe" {scope_attribute} Tag="" Version="1.0.0" ProviderKey="{{00000000-0000-0000-0000-000000000000}}">{ARP}</Registration>"#
        ))
        .unwrap()
    }

    #[test]
    fn wix7_scope() {
        for (value, expected) in [
            ("perMachine", WixBundleScope::Machine),
            ("perMachineOrUser", WixBundleScope::MachineOrUser),
            ("perUserOrMachine", WixBundleScope::UserOrMachine),
            ("perUser", WixBundleScope::User),
        ] {
            assert_eq!(
                registration(&format!(r#"Scope="{value}""#)).scope(),
                expected
            );
        }
    }

    #[test]
    fn legacy_per_machine() {
        assert_eq!(
            registration(r#"PerMachine="yes""#).scope(),
            WixBundleScope::Machine
        );
        assert_eq!(
            registration(r#"PerMachine="no""#).scope(),
            WixBundleScope::User
        );
        assert_eq!(registration("").scope(), WixBundleScope::User);
    }
}
