use crate::prompts::prompt::Prompt;
use derive_more::{AsRef, Deref, Display};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(AsRef, Clone, Default, Deref, Display, Deserialize, Serialize, Debug)]
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
    const DISALLOWED_CHARACTERS: [char; 9] = ['\\', '/', ':', '*', '?', '\"', '<', '>', '|'];

    pub fn parse(identifier: &str) -> Result<Self, PackageIdentifierError> {
        let chars_count = identifier.chars().count();
        if chars_count < Self::MIN_LENGTH as usize || chars_count > Self::MAX_LENGTH as usize {
            return Err(PackageIdentifierError::InvalidLength);
        }

        if identifier.chars().any(char::is_whitespace) {
            return Err(PackageIdentifierError::ContainsWhitespace);
        }

        if identifier.chars().any(char::is_control) {
            return Err(PackageIdentifierError::ContainsControlChars);
        }

        let mut parts_count: u8 = 0;
        for part in identifier.split('.') {
            parts_count += 1;
            if part
                .chars()
                .any(|char| Self::DISALLOWED_CHARACTERS.contains(&char))
            {
                return Err(PackageIdentifierError::DisallowedCharacters);
            }

            let chars_count = part.chars().count();
            if chars_count < Self::MIN_PART_LENGTH as usize
                || chars_count > Self::MAX_PART_LENGTH as usize
            {
                return Err(PackageIdentifierError::InvalidPartLength);
            }
        }

        if !(Self::MIN_PARTS..=Self::MAX_PARTS).contains(&parts_count) {
            return Err(PackageIdentifierError::InvalidPartCount);
        }

        Ok(Self(identifier.to_string()))
    }
}

impl Prompt for PackageIdentifier {
    const MESSAGE: &'static str = "软件包标识符:";
    const HELP_MESSAGE: Option<&'static str> =
        Some("软件包标识符的格式为 发布者.软件包名称");
    const PLACEHOLDER: Option<&'static str> = Some("Package.Identifier");
}

impl FromStr for PackageIdentifier {
    type Err = PackageIdentifierError;

    fn from_str(input: &str) -> Result<Self, PackageIdentifierError> {
        Self::parse(input)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PackageIdentifierError {
    #[error(
        "软件包标识符长度必须在 {} 和 {} 之间",
        PackageIdentifier::MIN_LENGTH,
        PackageIdentifier::MAX_LENGTH
    )]
    InvalidLength,
    #[error("软件包标识符不得包含空白")]
    ContainsWhitespace,
    #[error("软件包标识符不得包含任何控制字符")]
    ContainsControlChars,
    #[error(
        "软件包标识符不得包含以下任何字符: {:?}",
        PackageIdentifier::DISALLOWED_CHARACTERS
    )]
    DisallowedCharacters,
    #[error(
        "软件包标识符最少有 {} 部分，最多只能有 {} 部分",
        PackageIdentifier::MIN_PART_LENGTH,
        PackageIdentifier::MAX_PART_LENGTH
    )]
    InvalidPartLength, // 一样？
    #[error(
        "软件包标识符最少有 {} 部分，最多只能有 {} 部分",
        PackageIdentifier::MIN_PARTS,
        PackageIdentifier::MAX_PARTS
    )]
    InvalidPartCount,
}

#[cfg(test)]
mod tests {
    use crate::types::package_identifier::{PackageIdentifier, PackageIdentifierError};
    use const_format::str_repeat;
    use std::iter::repeat_n;

    // Replace itertools::intersperse with std::iter::Intersperse once it's stabilised

    #[test]
    fn test_valid_package_identifier() {
        assert!(PackageIdentifier::parse("Package.Identifier").is_ok());
    }

    #[test]
    fn test_too_long_package_identifier() {
        const NUM_DELIMITERS: u8 = PackageIdentifier::MAX_PARTS - 1;
        const PART_LEN: u8 =
            (PackageIdentifier::MAX_LENGTH - NUM_DELIMITERS).div_ceil(PackageIdentifier::MAX_PARTS);
        const PART: &str = str_repeat!("a", PART_LEN as usize);

        let identifier =
            itertools::intersperse(repeat_n(PART, PackageIdentifier::MAX_PARTS as usize), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::parse(&identifier).err(),
            Some(PackageIdentifierError::InvalidLength)
        );
    }

    #[test]
    fn test_too_many_parts_package_identifier() {
        let identifier = itertools::intersperse(
            repeat_n('a', PackageIdentifier::MAX_PARTS as usize + 1),
            '.',
        )
        .collect::<String>();

        assert_eq!(
            PackageIdentifier::parse(&identifier).err(),
            Some(PackageIdentifierError::InvalidPartCount)
        );
    }

    #[test]
    fn test_package_identifier_parts_too_long() {
        const PART: &str = str_repeat!("a", PackageIdentifier::MAX_PART_LENGTH as usize + 1);

        let identifier =
            itertools::intersperse(repeat_n(PART, PackageIdentifier::MIN_PARTS as usize), ".")
                .collect::<String>();

        assert_eq!(
            PackageIdentifier::parse(&identifier).err(),
            Some(PackageIdentifierError::InvalidPartLength)
        )
    }

    #[test]
    fn test_too_few_parts_package_identifier() {
        const IDENTIFIER: &str = str_repeat!("a", PackageIdentifier::MIN_LENGTH as usize);

        assert_eq!(
            PackageIdentifier::parse(IDENTIFIER).err(),
            Some(PackageIdentifierError::InvalidPartCount)
        )
    }

    #[test]
    fn test_package_identifier_contains_whitespace() {
        assert_eq!(
            PackageIdentifier::parse("Publisher.Pack age").err(),
            Some(PackageIdentifierError::ContainsWhitespace)
        );
    }

    #[test]
    fn test_package_identifier_contains_control_chars() {
        assert_eq!(
            PackageIdentifier::parse("Publisher.Pack\0age").err(),
            Some(PackageIdentifierError::ContainsControlChars)
        );
    }

    #[test]
    fn test_package_identifier_disallowed_characters() {
        for char in PackageIdentifier::DISALLOWED_CHARACTERS {
            let identifier = format!("Publisher.Pack{char}age");

            assert_eq!(
                PackageIdentifier::parse(&identifier).err(),
                Some(PackageIdentifierError::DisallowedCharacters)
            )
        }
    }
}
