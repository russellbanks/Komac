use std::str::FromStr;

use derive_more::{Deref, Display};
use serde::Serialize;
use serde_with::DeserializeFromStr;
use thiserror::Error;

use crate::shared::DISALLOWED_CHARACTERS;

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
    Serialize,
    DeserializeFromStr,
)]
pub struct PackageIdentifier(String);

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PackageIdentifierError {
    #[error("Package identifier cannot be empty")]
    Empty,
    #[error(
        "Package identifier cannot be more than {} characters long",
        PackageIdentifier::MAX_LENGTH
    )]
    TooLong,
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
        "The length of a part in a package identifier cannot be more than {} characters long",
        PackageIdentifier::MAX_PART_LENGTH
    )]
    PartTooLong,
    #[error(
        "The number of parts in the package identifier must be between {} and {}",
        PackageIdentifier::MIN_PARTS,
        PackageIdentifier::MAX_PARTS
    )]
    InvalidPartCount,
}

/// A Package Identifier parser and validator modelled off the regex pattern:
/// ^[^.\s\\/:*?"<>|\x01-\x1f]{1,32}(\.[^.\s\\/:*?"<>|\x01-\x1f]{1,32}){1,7}$
impl PackageIdentifier {
    const MAX_LENGTH: usize = 128;
    const MIN_PARTS: usize = 2;
    const MAX_PARTS: usize = 8;
    const MAX_PART_LENGTH: usize = 32;

    pub fn new(input: &str) -> Result<Self, PackageIdentifierError> {
        if input.is_empty() {
            return Err(PackageIdentifierError::Empty);
        } else if input.chars().count() > Self::MAX_LENGTH {
            return Err(PackageIdentifierError::TooLong);
        } else if input.contains(DISALLOWED_CHARACTERS) {
            return Err(PackageIdentifierError::DisallowedCharacters);
        } else if input.contains(char::is_whitespace) {
            return Err(PackageIdentifierError::ContainsWhitespace);
        } else if input.contains(char::is_control) {
            return Err(PackageIdentifierError::ContainsControlChars);
        }

        let mut parts_count = 0;
        for part in input.split('.') {
            parts_count += 1;
            if part.contains(DISALLOWED_CHARACTERS) {
                return Err(PackageIdentifierError::DisallowedCharacters);
            }

            if part.chars().count() > Self::MAX_PART_LENGTH {
                return Err(PackageIdentifierError::PartTooLong);
            }
        }

        if !(Self::MIN_PARTS..=Self::MAX_PARTS).contains(&parts_count) {
            return Err(PackageIdentifierError::InvalidPartCount);
        }

        Ok(Self(input.to_string()))
    }
}

impl FromStr for PackageIdentifier {
    type Err = PackageIdentifierError;

    fn from_str(s: &str) -> Result<Self, PackageIdentifierError> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use std::{iter::repeat_n, str::FromStr};

    use crate::shared::{
        DISALLOWED_CHARACTERS,
        package_identifier::{PackageIdentifier, PackageIdentifierError},
    };

    #[test]
    fn valid_package_identifier() {
        assert!(PackageIdentifier::new("Package.Identifier").is_ok());
    }

    #[test]
    fn too_long_package_identifier() {
        let num_delimiters = PackageIdentifier::MAX_PARTS - 1;
        let part_length =
            (PackageIdentifier::MAX_LENGTH - num_delimiters).div_ceil(PackageIdentifier::MAX_PARTS);

        let part = "a".repeat(part_length);

        let identifier =
            itertools::intersperse(repeat_n(&*part, PackageIdentifier::MAX_PARTS), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err(),
            Some(PackageIdentifierError::TooLong)
        );
    }

    #[test]
    fn too_many_parts_package_identifier() {
        let identifier =
            itertools::intersperse(repeat_n('a', PackageIdentifier::MAX_PARTS + 1), '.')
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err(),
            Some(PackageIdentifierError::InvalidPartCount)
        );
    }

    #[test]
    fn package_identifier_parts_too_long() {
        let part = "a".repeat(PackageIdentifier::MAX_PART_LENGTH as usize + 1);

        let identifier =
            itertools::intersperse(repeat_n(&*part, PackageIdentifier::MIN_PARTS), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::new(&identifier).err().unwrap(),
            PackageIdentifierError::PartTooLong
        )
    }

    #[test]
    fn too_few_parts_package_identifier() {
        let identifier = "a".repeat(PackageIdentifier::MIN_PARTS - 1);

        assert_eq!(
            PackageIdentifier::from_str(&identifier).err().unwrap(),
            PackageIdentifierError::InvalidPartCount
        )
    }

    #[test]
    fn package_identifier_contains_whitespace() {
        assert_eq!(
            PackageIdentifier::from_str("Publisher.Pack age")
                .err()
                .unwrap(),
            PackageIdentifierError::ContainsWhitespace
        );
    }

    #[test]
    fn package_identifier_contains_control_chars() {
        assert_eq!(
            PackageIdentifier::from_str("Publisher.Pack\0age")
                .err()
                .unwrap(),
            PackageIdentifierError::ContainsControlChars
        );
    }

    #[test]
    fn package_identifier_disallowed_characters() {
        for char in DISALLOWED_CHARACTERS {
            let identifier = format!("Publisher.Pack{char}age");

            assert_eq!(
                PackageIdentifier::from_str(&identifier).err().unwrap(),
                PackageIdentifierError::DisallowedCharacters
            )
        }
    }
}
