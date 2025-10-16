mod property;
mod provides;
mod related_package;

use std::collections::HashMap;

pub use property::MsiProperty;
pub use provides::Provides;
use related_package::RelatedPackage;
use serde::Deserialize;
use winget_types::Version;

use super::{super::package::PackageBase, Value};

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L355>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MsiPackage {
    #[serde(flatten)]
    pub base: PackageBase,
    #[serde(rename = "@ProductCode")]
    product_code: String,
    #[serde(rename = "@Language")]
    pub product_language: String,
    #[serde(rename = "@Version")]
    version: Version,
    #[serde(rename = "@UpgradeCode")]
    upgrade_code: Option<String>,
    #[serde(rename = "MsiProperty", default)]
    pub properties: Vec<MsiProperty>,
    #[serde(default)]
    pub provides: Vec<Provides>,
    #[serde(default)]
    pub related_package: Vec<RelatedPackage>,
}

impl MsiPackage {
    #[inline]
    pub const fn id(&self) -> &str {
        self.base.id()
    }

    #[inline]
    pub const fn product_code(&self) -> &str {
        self.product_code.as_str()
    }

    #[inline]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[inline]
    pub fn upgrade_code(&self) -> Option<&str> {
        self.upgrade_code.as_deref()
    }

    /// Returns true if any of this package's properties are [`ARPSYSTEMCOMPONENT`] with a value
    /// of 1.
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
