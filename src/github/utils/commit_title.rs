use std::fmt;

use winget_types::{PackageIdentifier, PackageVersion};

use super::UpdateState;

pub struct CommitTitle<'a> {
    package_identifier: &'a PackageIdentifier,
    package_version: &'a PackageVersion,
    update_state: UpdateState,
}

impl<'a> CommitTitle<'a> {
    #[inline]
    pub const fn new(
        package_identifier: &'a PackageIdentifier,
        package_version: &'a PackageVersion,
        update_state: UpdateState,
    ) -> Self {
        Self {
            package_identifier,
            package_version,
            update_state,
        }
    }

    pub const fn remove(
        package_identifier: &'a PackageIdentifier,
        package_version: &'a PackageVersion,
    ) -> Self {
        Self::new(
            package_identifier,
            package_version,
            UpdateState::RemoveVersion,
        )
    }
}

impl fmt::Display for CommitTitle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{update_state}: {package_identifier} version {package_version}",
            update_state = self.update_state,
            package_identifier = self.package_identifier,
            package_version = self.package_version
        )
    }
}
