use std::{cmp::max, collections::BTreeSet, fmt};

use winget_types::PackageVersion;

#[derive(Copy, Clone)]
pub enum UpdateState {
    NewPackage,
    NewVersion,
    AddVersion,
    UpdateVersion,
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

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NewPackage => "New package",
            Self::NewVersion => "New version",
            Self::AddVersion => "Add version",
            Self::UpdateVersion => "Update version",
            Self::RemoveVersion => "Remove version",
        }
    }
}

impl fmt::Display for UpdateState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
