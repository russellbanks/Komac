use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    installer::installer_type::InstallerType,
    locale::DefaultLocaleManifest,
    shared::{PackageVersion, Version},
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct AppsAndFeaturesEntry {
    pub display_name: Option<String>,
    pub publisher: Option<String>,
    pub display_version: Option<Version>,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub installer_type: Option<InstallerType>,
}

impl AppsAndFeaturesEntry {
    #[must_use]
    pub const fn is_any_some(&self) -> bool {
        self.display_name.is_some()
            || self.publisher.is_some()
            || self.display_version.is_some()
            || self.product_code.is_some()
            || self.upgrade_code.is_some()
            || self.installer_type.is_some()
    }

    pub fn deduplicate(
        &mut self,
        package_version: &PackageVersion,
        locale_manifest: &DefaultLocaleManifest,
    ) {
        if self.display_name.as_deref() == Some(&***locale_manifest.package_name) {
            self.display_name = None;
        }
        if self.publisher.as_deref() == Some(&**locale_manifest.publisher) {
            self.publisher = None;
        }
        if self.display_version.as_ref() == Some(&**package_version) {
            self.display_version = None;
        }
    }
}
