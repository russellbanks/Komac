use crate::prompts::prompt::Prompt;
use crate::types::DISALLOWED_CHARACTERS;
use derive_more::{AsRef, Deref, Display};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PackageIdentifierError {
    #[error(
        "Package identifier length must be between {} and {} characters",
        PackageIdentifier::MIN_LENGTH,
        PackageIdentifier::MAX_LENGTH
    )]
    InvalidLength,
    #[error("Package identifier may not contain whitespace")]
    ContainsWhitespace,
    #[error("Package identifier may not contain any control characters")]
    ContainsControlChars,
    #[error(
        "Package identifier may not contain any of the following characters: {:?}",
        DISALLOWED_CHARACTERS
    )]
    DisallowedCharacters,
    #[error(
        "The number of parts in the package identifier must be between {} and {}",
        PackageIdentifier::MIN_PART_LENGTH,
        PackageIdentifier::MAX_PART_LENGTH
    )]
    InvalidPartLength,
    #[error(
        "The number of parts in the package identifier must be between {} and {}",
        PackageIdentifier::MIN_PARTS,
        PackageIdentifier::MAX_PARTS
    )]
    InvalidPartCount,
}

#[derive(
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DeserializeFromStr,
    SerializeDisplay,
)]
pub struct PackageIdentifier(String);

/// A Package Identifier parser and validator modelled off the regex pattern:
/// ^[^.\s\\/:*?"<>|\x01-\x1f]{1,32}(\.[^.\s\\/:*?"<>|\x01-\x1f]{1,32}){1,7}$
impl PackageIdentifier {
    const MIN_LENGTH: u8 = 1;
    const MAX_LENGTH: u8 = 128;
    const MIN_PARTS: u8 = 2;
    const MAX_PARTS: u8 = 8;
    const MIN_PART_LENGTH: u8 = 1;
    const MAX_PART_LENGTH: u8 = 32;
}

impl FromStr for PackageIdentifier {
    type Err = PackageIdentifierError;

    fn from_str(input: &str) -> Result<Self, PackageIdentifierError> {
        if !(Self::MIN_LENGTH as usize..=Self::MAX_LENGTH as usize).contains(&input.chars().count())
        {
            return Err(PackageIdentifierError::InvalidLength);
        }

        if input.contains(DISALLOWED_CHARACTERS) {
            return Err(PackageIdentifierError::DisallowedCharacters);
        }

        if input.contains(char::is_whitespace) {
            return Err(PackageIdentifierError::ContainsWhitespace);
        }

        if input.contains(char::is_control) {
            return Err(PackageIdentifierError::ContainsControlChars);
        }

        let mut parts_count: u8 = 0;
        for part in input.split('.') {
            parts_count += 1;
            if part.contains(DISALLOWED_CHARACTERS) {
                return Err(PackageIdentifierError::DisallowedCharacters);
            }

            if !(Self::MIN_PART_LENGTH as usize..=Self::MAX_PART_LENGTH as usize)
                .contains(&part.chars().count())
            {
                return Err(PackageIdentifierError::InvalidPartLength);
            }
        }

        if !(Self::MIN_PARTS..=Self::MAX_PARTS).contains(&parts_count) {
            return Err(PackageIdentifierError::InvalidPartCount);
        }

        Ok(Self(input.to_string()))
    }
}

impl Prompt for PackageIdentifier {
    const MESSAGE: &'static str = "Package identifier:";
    const HELP_MESSAGE: Option<&'static str> =
        Some("Package Identifiers are in the format of Package.Identifier");
    const PLACEHOLDER: Option<&'static str> = Some("Package.Identifier");
}

#[cfg(test)]
mod tests {
    use crate::types::package_identifier::{PackageIdentifier, PackageIdentifierError};
    use crate::types::DISALLOWED_CHARACTERS;
    use const_format::str_repeat;
    use std::iter::repeat_n;
    use std::str::FromStr;
    // Replace itertools::intersperse with std::iter::Intersperse once it's stabilised

    #[test]
    fn valid_package_identifier() {
        assert!(PackageIdentifier::from_str("Package.Identifier").is_ok());
    }

    #[test]
    fn too_long_package_identifier() {
        const NUM_DELIMITERS: u8 = PackageIdentifier::MAX_PARTS - 1;
        const PART_LEN: u8 =
            (PackageIdentifier::MAX_LENGTH - NUM_DELIMITERS).div_ceil(PackageIdentifier::MAX_PARTS);
        const PART: &str = str_repeat!("a", PART_LEN as usize);

        let identifier =
            itertools::intersperse(repeat_n(PART, PackageIdentifier::MAX_PARTS as usize), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err(),
            Some(PackageIdentifierError::InvalidLength)
        );
    }

    #[test]
    fn too_many_parts_package_identifier() {
        let identifier = itertools::intersperse(
            repeat_n('a', PackageIdentifier::MAX_PARTS as usize + 1),
            '.',
        )
        .collect::<String>();

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err(),
            Some(PackageIdentifierError::InvalidPartCount)
        );
    }

    #[test]
    fn package_identifier_parts_too_long() {
        const PART: &str = str_repeat!("a", PackageIdentifier::MAX_PART_LENGTH as usize + 1);

        let identifier =
            itertools::intersperse(repeat_n(PART, PackageIdentifier::MIN_PARTS as usize), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err(),
            Some(PackageIdentifierError::InvalidPartLength)
        )
    }

    #[test]
    fn too_few_parts_package_identifier() {
        const IDENTIFIER: &str = str_repeat!("a", PackageIdentifier::MIN_LENGTH as usize);

        assert_eq!(
            PackageIdentifier::from_str(IDENTIFIER).err(),
            Some(PackageIdentifierError::InvalidPartCount)
        )
    }

    #[test]
    fn package_identifier_contains_whitespace() {
        assert_eq!(
            PackageIdentifier::from_str("Publisher.Pack age").err(),
            Some(PackageIdentifierError::ContainsWhitespace)
        );
    }

    #[test]
    fn package_identifier_contains_control_chars() {
        assert_eq!(
            PackageIdentifier::from_str("Publisher.Pack\0age").err(),
            Some(PackageIdentifierError::ContainsControlChars)
        );
    }

    #[test]
    fn package_identifier_disallowed_characters() {
        for char in DISALLOWED_CHARACTERS {
            let identifier = format!("Publisher.Pack{char}age");

            assert_eq!(
                PackageIdentifier::from_str(&identifier).err(),
                Some(PackageIdentifierError::DisallowedCharacters)
            )
        }
    }
}
