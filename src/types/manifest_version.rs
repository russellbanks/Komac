use color_eyre::eyre::OptionExt;
use const_format::{writec, ConstDebug, Error, Formatter};
use derive_more::Display;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

#[derive(SerializeDisplay, DeserializeFromStr, Display, ConstDebug)]
#[display("{_0}.{_1}.{_2}")]
pub struct ManifestVersion(u16, u16, u16);

impl Default for ManifestVersion {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl FromStr for ManifestVersion {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(Self::PARTS_COUNT as usize, Self::SEPARATOR);
        let major = parts
            .next()
            .ok_or_eyre("No major version")?
            .parse::<u16>()?;
        let minor = parts
            .next()
            .ok_or_eyre("No minor version")?
            .parse::<u16>()?;
        let patch = parts
            .next()
            .ok_or_eyre("No patch version")?
            .parse::<u16>()?;
        Ok(Self(major, minor, patch))
    }
}

impl ManifestVersion {
    pub const DEFAULT: Self = Self(1, 9, 0);
    const PARTS_COUNT: u8 = 3;
    const SEPARATOR: char = '.';

    pub const fn const_display_fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writec!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
