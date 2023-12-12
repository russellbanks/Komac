use crate::types::package_version::PackageVersion;
use std::cmp::max;
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

pub fn get_update_state(
    version: &PackageVersion,
    versions: &[PackageVersion],
    latest_version: &PackageVersion,
) -> UpdateState {
    match version {
        version if versions.contains(version) => UpdateState::UpdateVersion,
        version if max(version, latest_version) == version => UpdateState::NewVersion,
        _ => UpdateState::AddVersion,
    }
}
