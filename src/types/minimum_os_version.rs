use crate::types::version::Version;
use color_eyre::eyre::OptionExt;
use derive_more::Display;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

#[derive(
    SerializeDisplay,
    DeserializeFromStr,
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
#[display("{_0}.{_1}.{_2}.{_3}")]
pub struct MinimumOSVersion(pub u16, pub u16, pub u16, pub u16);

impl FromStr for MinimumOSVersion {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(Self::MAX_PARTS as usize, Version::SEPARATOR);
        let major = parts
            .next()
            .ok_or_eyre("No major version")?
            .parse::<u16>()?;
        let minor = parts.next().map_or(Ok(0), u16::from_str)?;
        let patch = parts.next().map_or(Ok(0), u16::from_str)?;
        let build = parts.next().map_or(Ok(0), u16::from_str)?;
        Ok(Self(major, minor, patch, build))
    }
}

impl MinimumOSVersion {
    const MAX_PARTS: u8 = 4;

    pub const fn removable() -> Self {
        Self(10, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::minimum_os_version::MinimumOSVersion;
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case("10.0.17763.0", MinimumOSVersion(10, 0, 17763, 0))]
    #[case("11", MinimumOSVersion(11, 0, 0, 0))]
    #[case("10.1", MinimumOSVersion(10, 1, 0, 0))]
    #[case("0", MinimumOSVersion(0, 0, 0, 0))]
    #[case(
        "65535.65535.65535.65535",
        MinimumOSVersion(u16::MAX, u16::MAX, u16::MAX, u16::MAX)
    )]
    fn from_str(#[case] minimum_os_version: &str, #[case] expected: MinimumOSVersion) {
        assert_eq!(
            MinimumOSVersion::from_str(minimum_os_version).unwrap(),
            expected
        )
    }

    #[test]
    fn display() {
        assert_eq!(MinimumOSVersion(1, 2, 3, 4).to_string(), "1.2.3.4")
    }
}
