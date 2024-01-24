use crate::types::package_version::PackageVersion;
use std::cmp::max;
use std::collections::BTreeSet;
use strum::Display;

#[derive(Display)]
pub enum UpdateState {
    #[strum(serialize = "New package")]
    NewPackage,
    #[strum(serialize = "New version")]
    NewVersion,
    #[strum(serialize = "Add version")]
    AddVersion,
    #[strum(serialize = "Update version")]
    UpdateVersion,
    #[strum(serialize = "Remove version")]
    RemoveVersion,
}
impl UpdateState {
    pub fn get(
        version: &PackageVersion,
        versions: Option<&BTreeSet<PackageVersion>>,
        latest_version: Option<&PackageVersion>,
    ) -> Self {
        match version {
            version if versions.map_or(false, |versions| versions.contains(version)) => {
                Self::UpdateVersion
            }
            version if latest_version.map_or(false, |latest| max(version, latest) == version) => {
                Self::NewVersion
            }
            _ if versions.is_none() => Self::NewPackage,
            _ => Self::AddVersion,
        }
    }
}
