use std::{cmp::max, collections::BTreeSet};

use derive_more::Display;
use winget_types::PackageVersion;

#[derive(Copy, Clone, Display)]
pub enum UpdateState {
    #[display("New package")]
    NewPackage,
    #[display("New version")]
    NewVersion,
    #[display("Add version")]
    AddVersion,
    #[display("Update version")]
    UpdateVersion,
    #[display("Remove version")]
    RemoveVersion,
}
impl UpdateState {
    pub fn get(version: &PackageVersion, versions: Option<&BTreeSet<PackageVersion>>) -> Self {
        match version {
            version if versions.is_some_and(|versions| versions.contains(version)) => {
                Self::UpdateVersion
            }
            version
                if versions
                    .and_then(BTreeSet::last)
                    .is_some_and(|latest| max(version, latest) == version) =>
            {
                Self::NewVersion
            }
            _ if versions.is_none() => Self::NewPackage,
            _ => Self::AddVersion,
        }
    }
}
