use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::shared::{PackageIdentifier, PackageVersion};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Dependencies {
    pub windows_features: Option<BTreeSet<String>>,
    pub windows_libraries: Option<BTreeSet<String>>,
    #[serde(rename = "PackageDependencies")]
    pub package: Option<BTreeSet<PackageDependencies>>,
    #[serde(rename = "ExternalDependencies")]
    pub external: Option<BTreeSet<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct PackageDependencies {
    pub package_identifier: PackageIdentifier,
    pub minimum_version: Option<PackageVersion>,
}
