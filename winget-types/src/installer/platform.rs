use std::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};

use bitflags::bitflags;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};
use thiserror::Error;

bitflags! {
    /// A list of installer supported operating systems internally represented as bit flags
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Platform: u8 {
        const WINDOWS_DESKTOP = 1;
        const WINDOWS_UNIVERSAL = 1 << 1;
    }
}

const WINDOWS_DESKTOP: &str = "Windows.Desktop";
const WINDOWS_UNIVERSAL: &str = "Windows.Universal";

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::WINDOWS_DESKTOP => f.write_str(WINDOWS_DESKTOP),
            Self::WINDOWS_UNIVERSAL => f.write_str(WINDOWS_UNIVERSAL),
            _ => bitflags::parser::to_writer(self, f),
        }
    }
}

#[derive(Error, Debug, Eq, PartialEq)]
#[error("Platform did not match either `{WINDOWS_DESKTOP}` or `{WINDOWS_UNIVERSAL}`")]
pub struct PlatformParseError;

impl FromStr for Platform {
    type Err = PlatformParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            WINDOWS_DESKTOP => Ok(Self::WINDOWS_DESKTOP),
            WINDOWS_UNIVERSAL => Ok(Self::WINDOWS_UNIVERSAL),
            _ => Err(PlatformParseError),
        }
    }
}

impl Serialize for Platform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.iter().count()))?;
        for platform in self.iter() {
            match platform {
                Self::WINDOWS_DESKTOP => seq.serialize_element(WINDOWS_DESKTOP)?,
                Self::WINDOWS_UNIVERSAL => seq.serialize_element(WINDOWS_UNIVERSAL)?,
                _ => {}
            }
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Platform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PlatformVisitor;

        impl<'de> Visitor<'de> for PlatformVisitor {
            type Value = Platform;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of platform strings")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut platform = Platform::empty();

                while let Some(value) = seq.next_element::<&str>()? {
                    match value {
                        WINDOWS_DESKTOP => platform |= Platform::WINDOWS_DESKTOP,
                        WINDOWS_UNIVERSAL => platform |= Platform::WINDOWS_UNIVERSAL,
                        _ => {
                            return Err(serde::de::Error::unknown_variant(
                                value,
                                &[WINDOWS_DESKTOP, WINDOWS_UNIVERSAL],
                            ));
                        }
                    }
                }

                Ok(platform)
            }
        }

        deserializer.deserialize_seq(PlatformVisitor)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use crate::installer::{Platform, platform::PlatformParseError};

    #[rstest]
    #[case(
        Platform::all(),
        indoc! {"
            - Windows.Desktop
            - Windows.Universal
        "}
    )]
    #[case(
        Platform::empty(),
        indoc! {"
            []
        "}
    )]
    #[case(
        Platform::WINDOWS_DESKTOP,
        indoc! {"
            - Windows.Desktop
        "}
    )]
    #[case(
        Platform::WINDOWS_UNIVERSAL,
        indoc! {"
            - Windows.Universal
        "}
    )]
    fn serialize_platform(#[case] platform: Platform, #[case] expected: &str) {
        assert_eq!(serde_yaml::to_string(&platform).unwrap(), expected);
    }

    #[rstest]
    #[case(
        indoc! {"
            - Windows.Desktop
            - Windows.Universal
        "},
        Platform::all(),
    )]
    #[case(
        indoc! {"
            []
        "},
        Platform::empty()
    )]
    #[case(
        indoc! {"
            - Windows.Desktop
        "},
        Platform::WINDOWS_DESKTOP
    )]
    #[case(
        indoc! {"
            - Windows.Universal
        "},
        Platform::WINDOWS_UNIVERSAL
    )]
    fn deserialize_platform(#[case] input: &str, #[case] expected: Platform) {
        assert_eq!(serde_yaml::from_str::<Platform>(input).unwrap(), expected);
    }

    #[test]
    fn platform_serialize_ordered() {
        let input = indoc! {"
            - Windows.Universal
            - Windows.Desktop
        "};

        let deserialized = serde_yaml::from_str::<Platform>(input).unwrap();

        assert_eq!(
            serde_yaml::to_string(&deserialized).unwrap(),
            indoc! {"
                - Windows.Desktop
                - Windows.Universal
            "}
        );
    }

    #[test]
    fn from_str() {
        assert!("Windows.Desktop".parse::<Platform>().is_ok());
        assert!("Windows.Universal".parse::<Platform>().is_ok());
        assert_eq!(
            "WindowsDesktop".parse::<Platform>().err().unwrap(),
            PlatformParseError
        );
    }
}
