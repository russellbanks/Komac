use std::{num::ParseIntError, str::FromStr};

use derive_more::Display;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::shared::Version;

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[display("{_0}.{_1}.{_2}.{_3}")]
pub struct MinimumOSVersion(u16, u16, u16, u16);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum MinimumOSVersionError {
    #[error("Minimum OS version must have at least a major version part")]
    NoVersionParts,
    #[error(transparent)]
    InvalidPart(#[from] ParseIntError),
}

impl MinimumOSVersion {
    const MAX_PARTS: u8 = 4;

    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16, build: u16) -> Self {
        Self(major, minor, patch, build)
    }

    #[must_use]
    pub const fn removable() -> Self {
        Self(10, 0, 0, 0)
    }
}

impl FromStr for MinimumOSVersion {
    type Err = MinimumOSVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(Self::MAX_PARTS as usize, Version::SEPARATOR);

        let major = parts
            .next()
            .ok_or(MinimumOSVersionError::NoVersionParts)?
            .parse::<u16>()?;
        let minor = parts.next().map_or(Ok(0), u16::from_str)?;
        let patch = parts.next().map_or(Ok(0), u16::from_str)?;
        let build = parts.next().map_or(Ok(0), u16::from_str)?;

        Ok(Self(major, minor, patch, build))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::installer::minimum_os_version::MinimumOSVersion;

    #[rstest]
    #[case("10.0.17763.0", MinimumOSVersion(10, 0, 17763, 0))]
    #[case("11", MinimumOSVersion(11, 0, 0, 0))]
    #[case("10.1", MinimumOSVersion(10, 1, 0, 0))]
    #[case("0", MinimumOSVersion(0, 0, 0, 0))]
    #[case(
        "65535.65535.65535.65535",
        MinimumOSVersion(u16::MAX, u16::MAX, u16::MAX, u16::MAX)
    )]
    fn valid_minimum_os_version(
        #[case] minimum_os_version: &str,
        #[case] expected: MinimumOSVersion,
    ) {
        assert_eq!(
            minimum_os_version.parse::<MinimumOSVersion>().unwrap(),
            expected
        )
    }

    #[test]
    fn minimum_os_version_display() {
        let version = "1.2.3.4";

        assert_eq!(MinimumOSVersion(1, 2, 3, 4).to_string(), version);

        // Test round tripping
        assert_eq!(
            version.parse::<MinimumOSVersion>().unwrap().to_string(),
            version
        );
    }
}
