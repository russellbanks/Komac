use crate::prompts::prompt::Prompt;
use crate::types::version::Version;
use crate::types::DISALLOWED_CHARACTERS;
use derive_more::{Deref, Display};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PackageVersionError {
    #[error("Package version may not contain any control characters")]
    ContainsControlChars,
    #[error(
        "Package version may not contain any of the following characters: {:?}",
        DISALLOWED_CHARACTERS
    )]
    DisallowedCharacters,
}

#[derive(
    SerializeDisplay,
    DeserializeFromStr,
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct PackageVersion(Version);

impl FromStr for PackageVersion {
    type Err = PackageVersionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains(DISALLOWED_CHARACTERS) {
            return Err(PackageVersionError::DisallowedCharacters);
        }

        if input.contains(char::is_control) {
            return Err(PackageVersionError::ContainsControlChars);
        }

        Ok(Self(Version::new(input)))
    }
}

impl Prompt for PackageVersion {
    const MESSAGE: &'static str = "Package version:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: 1.2.3");
    const PLACEHOLDER: Option<&'static str> = None;
}

#[cfg(test)]
mod tests {
    use crate::types::package_version::{PackageVersion, PackageVersionError};
    use crate::types::DISALLOWED_CHARACTERS;
    use std::str::FromStr;

    #[test]
    fn package_version_contains_control_chars() {
        assert_eq!(
            PackageVersion::from_str("1.2\03").err(),
            Some(PackageVersionError::ContainsControlChars)
        );
    }

    #[test]
    fn package_version_disallowed_characters() {
        for char in DISALLOWED_CHARACTERS {
            let version = format!("1.2{char}3");

            assert_eq!(
                PackageVersion::from_str(&version).err(),
                Some(PackageVersionError::DisallowedCharacters)
            )
        }
    }
}
