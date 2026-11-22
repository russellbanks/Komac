pub mod install_condition;
pub mod msi;

use std::collections::HashMap;

use install_condition::{InstallCondition, Value};
use msi::MsiPackage;
use serde::Deserialize;

use super::{BundleCacheType, bool_from_yes_no, deserialize_from_str};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Package {
    #[serde(rename = "BundlePackage")]
    Bundle(PackageBase),
    #[serde(rename = "ExePackage")]
    Exe(PackageBase),
    #[serde(rename = "MsiPackage")]
    Msi(Box<MsiPackage>),
    #[serde(rename = "MspPackage")]
    Msp(PackageBase),
    #[serde(rename = "MsuPackage")]
    Msu(PackageBase),
}

impl Package {
    #[expect(unused)]
    pub fn try_into_msi(self) -> Option<MsiPackage> {
        match self {
            Self::Msi(msi) => Some(*msi),
            _ => None,
        }
    }

    pub fn try_as_msi(&self) -> Option<&MsiPackage> {
        match self {
            Self::Msi(msi) => Some(msi),
            _ => None,
        }
    }
}

/// Attributes that are present in all package types
///
/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L355>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageBase {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Cache", default)]
    cache: BundleCacheType,
    #[serde(rename = "@CacheId")]
    cache_id: String,
    #[serde(rename = "@InstallSize", deserialize_with = "deserialize_from_str")]
    install_size: u32,
    #[serde(rename = "@Size", deserialize_with = "deserialize_from_str")]
    size: u32,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    per_machine: bool,
    #[serde(rename = "@Permanent", deserialize_with = "bool_from_yes_no", default)]
    permanent: bool,
    #[serde(rename = "@Vital", deserialize_with = "bool_from_yes_no", default)]
    vital: bool,
    #[serde(rename = "@RollbackBoundaryForward")]
    pub rollback_boundary_forward: Option<String>,
    #[serde(rename = "@RollbackBoundaryBackward")]
    pub rollback_boundary_backward: Option<String>,
    #[serde(rename = "@LogPathVariable")]
    pub log_path_variable: Option<String>,
    #[serde(rename = "@RollbackLogPathVariable")]
    pub rollback_log_path_variable: Option<String>,
    #[serde(rename = "@InstallCondition")]
    pub install_condition: Option<InstallCondition>,
    #[serde(rename = "@RepairCondition")]
    pub repair_condition: Option<String>,
}

impl PackageBase {
    /// Returns the package's ID.
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn cache(&self) -> BundleCacheType {
        self.cache
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn cache_id(&self) -> &str {
        self.cache_id.as_str()
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn install_size(&self) -> u32 {
        self.install_size
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn size(&self) -> u32 {
        self.size
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn per_machine(&self) -> bool {
        self.per_machine
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn permanent(&self) -> bool {
        self.permanent
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn vital(&self) -> bool {
        self.vital
    }

    pub fn evaluate_install_condition(&self, variables: &HashMap<&str, Value>) -> bool {
        self.install_condition
            .as_ref()
            .is_none_or(|condition| condition.evaluate(variables))
    }
}

#[cfg(test)]
mod tests {
    use super::Package;

    #[test]
    fn package_sizes() {
        let Package::Exe(exe) = quick_xml::de::from_str::<Package>(
            r#"<ExePackage Id="exe" CacheId="exe_cache" InstallSize="1234" Size="5678" />"#,
        )
        .unwrap() else {
            panic!("expected an ExePackage");
        };
        assert_eq!(exe.install_size, 1234);
        assert_eq!(exe.size, 5678);

        let Package::Msi(msi) = quick_xml::de::from_str::<Package>(
            r#"<MsiPackage Id="msi" CacheId="msi_cache" InstallSize="4321" Size="8765" ProductCode="{00000000-0000-0000-0000-000000000000}" Language="1033" Version="1.2.3" />"#,
        )
        .unwrap() else {
            panic!("expected an MsiPackage");
        };
        assert_eq!(msi.base.install_size, 4321);
        assert_eq!(msi.base.size, 8765);
    }
}
