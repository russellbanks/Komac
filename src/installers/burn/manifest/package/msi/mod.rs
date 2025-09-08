mod property;
mod provides;
mod related_package;

use std::collections::HashMap;

pub use property::MsiProperty;
use provides::Provides;
use related_package::RelatedPackage;
use serde::Deserialize;
use winget_types::Version;

use super::super::package::PackageBase;
use crate::installers::burn::manifest::install_condition::Value;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L355>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MsiPackage<'manifest> {
    #[serde(flatten, borrow)]
    pub base: PackageBase<'manifest>,
    #[serde(rename = "@ProductCode")]
    pub product_code: &'manifest str,
    #[serde(rename = "@Language")]
    pub product_language: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: Version,
    #[serde(rename = "@UpgradeCode")]
    pub upgrade_code: Option<&'manifest str>,
    #[serde(rename = "MsiProperty", borrow, default)]
    pub properties: Vec<MsiProperty<'manifest>>,
    #[serde(borrow)]
    pub provides: Provides<'manifest>,
    #[serde(default, borrow)]
    pub related_package: Vec<RelatedPackage<'manifest>>,
}

impl<'manifest> MsiPackage<'manifest> {
    #[expect(dead_code)]
    pub const fn id(&self) -> &'manifest str {
        self.base.id()
    }

    /// Returns true if any of this package's properties are [`ARPSYSTEMCOMPONENT`] with a value of
    /// 1.
    ///
    /// [`ARPSYSTEMCOMPONENT`]: https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
    pub fn is_arp_system_component(&self) -> bool {
        self.properties
            .iter()
            .any(MsiProperty::is_arp_system_component)
    }

    #[inline]
    pub fn evaluate_install_condition(&self, variables: &HashMap<&str, Value>) -> bool {
        self.base.evaluate_install_condition(variables)
    }
}
