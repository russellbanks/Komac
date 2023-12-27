use crate::manifest::MANIFEST_VERSION;
use nutype::nutype;
use std::str::FromStr;

#[nutype(
    validate(predicate = is_manifest_version_valid),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct ManifestVersion(String);

impl Default for ManifestVersion {
    fn default() -> Self {
        Self::new(MANIFEST_VERSION).unwrap()
    }
}

fn is_manifest_version_valid(input: &str) -> bool {
    let parts = input.split('.').collect::<Vec<_>>();

    if parts.len() != 3 {
        return false;
    }

    if parts.into_iter().any(|part| u16::from_str(part).is_err()) {
        return false;
    }

    true
}
