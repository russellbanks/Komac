pub mod install_condition;
pub mod msi;

use std::collections::HashMap;

use install_condition::{InstallCondition, Value};
use msi::MsiPackage;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use super::{BundleCacheType, bool_from_yes_no};

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Package<'manifest> {
    #[serde(rename = "BundlePackage", borrow)]
    Bundle(PackageBase<'manifest>),
    #[serde(rename = "ExePackage", borrow)]
    Exe(PackageBase<'manifest>),
    #[serde(rename = "MsiPackage", borrow)]
    Msi(MsiPackage<'manifest>),
    #[serde(rename = "MspPackage", borrow)]
    Msp(PackageBase<'manifest>),
    #[serde(rename = "MsuPackage", borrow)]
    Msu(PackageBase<'manifest>),
}

impl<'manifest> Package<'manifest> {
    #[inline]
    pub fn try_into_msi(self) -> Option<MsiPackage<'manifest>> {
        match self {
            Self::Msi(msi) => Some(msi),
            _ => None,
        }
    }
}

/// Attributes that are present in all package types
///
/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L355>
#[expect(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageBase<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@Cache", default)]
    pub cache: BundleCacheType,
    #[serde(rename = "@CacheId")]
    pub cache_id: &'manifest str,
    #[serde(rename = "@InstallSize")]
    #[serde_as(as = "DisplayFromStr")]
    pub install_size: u32,
    #[serde(rename = "@Size")]
    #[serde_as(as = "DisplayFromStr")]
    pub size: u32,
    #[serde(rename = "@PerMachine", deserialize_with = "bool_from_yes_no", default)]
    pub per_machine: bool,
    #[serde(rename = "@Permanent", deserialize_with = "bool_from_yes_no", default)]
    pub permanent: bool,
    #[serde(rename = "@Vital", deserialize_with = "bool_from_yes_no", default)]
    pub vital: bool,
    #[serde(rename = "@RollbackBoundaryForward")]
    pub rollback_boundary_forward: Option<&'manifest str>,
    #[serde(rename = "@RollbackBoundaryBackward")]
    pub rollback_boundary_backward: Option<&'manifest str>,
    #[serde(rename = "@LogPathVariable")]
    pub log_path_variable: Option<&'manifest str>,
    #[serde(rename = "@RollbackLogPathVariable")]
    pub rollback_log_path_variable: Option<&'manifest str>,
    #[serde(rename = "@InstallCondition")]
    pub install_condition: Option<InstallCondition>,
    #[serde(rename = "@RepairCondition")]
    pub repair_condition: Option<&'manifest str>,
}

impl PackageBase<'_> {
    pub fn evaluate_install_condition(&self, variables: &HashMap<&str, Value>) -> bool {
        self.install_condition
            .as_ref()
            .is_none_or(|condition| condition.evaluate(variables))
    }
}
