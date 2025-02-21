use std::str::FromStr;

use derive_more::{Deref, Display, Into};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::shared::{DISALLOWED_CHARACTERS, version::Version};

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Into,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[into(ref)]
pub struct PackageVersion(Version);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PackageVersionError {
    #[error("Package version may not contain any control characters")]
    ContainsControlChars,
    #[error(
        "Package version may not contain any of the following characters: {:?}",
        DISALLOWED_CHARACTERS
    )]
    DisallowedCharacters,
    #[error(
        "Package version cannot be more than {} characters long",
        PackageVersion::MAX_LENGTH
    )]
    TooLong,
}

impl PackageVersion {
    const MAX_LENGTH: usize = 1 << 7;

    pub fn new<S: AsRef<str>>(input: S) -> Result<Self, PackageVersionError> {
        let input = input.as_ref();

        if input.contains(DISALLOWED_CHARACTERS) {
            Err(PackageVersionError::DisallowedCharacters)
        } else if input.contains(char::is_control) {
            Err(PackageVersionError::ContainsControlChars)
        } else if input.chars().count() > Self::MAX_LENGTH {
            Err(PackageVersionError::TooLong)
        } else {
            Ok(Self(Version::new(input)))
        }
    }
}

impl FromStr for PackageVersion {
    type Err = PackageVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::{
        DISALLOWED_CHARACTERS,
        package_version::{PackageVersion, PackageVersionError},
    };

    #[test]
    fn package_version_disallowed_characters() {
        for char in DISALLOWED_CHARACTERS {
            assert_eq!(
                PackageVersion::new(format!("1.2{char}3")).err().unwrap(),
                PackageVersionError::DisallowedCharacters
            )
        }
    }

    #[test]
    fn package_version_contains_control_chars() {
        assert_eq!(
            PackageVersion::new("1.2\03").err().unwrap(),
            PackageVersionError::ContainsControlChars
        );
    }

    #[test]
    fn unicode_package_version_max_length() {
        let version = "🦀".repeat(PackageVersion::MAX_LENGTH);

        // Ensure that it's character length that's being checked and not byte or UTF-16 length
        assert!(version.len() > PackageVersion::MAX_LENGTH);
        assert!(version.encode_utf16().count() > PackageVersion::MAX_LENGTH);
        assert_eq!(version.chars().count(), PackageVersion::MAX_LENGTH);
        assert!(PackageVersion::new(version).is_ok());
    }

    #[test]
    fn package_version_too_long() {
        let version = "🦀".repeat(PackageVersion::MAX_LENGTH + 1);

        assert_eq!(
            PackageVersion::new(version).err().unwrap(),
            PackageVersionError::TooLong
        );
    }
}
