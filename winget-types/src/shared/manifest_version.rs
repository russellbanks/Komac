use std::{num::ParseIntError, str::FromStr};

use const_format::{ConstDebug, Error, Formatter, writec};
use derive_more::Display;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

#[derive(
    ConstDebug,
    Debug,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[display("{_0}.{_1}.{_2}")]
pub struct ManifestVersion(u16, u16, u16);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ManifestVersionError {
    #[error("Manifest version must have a major part")]
    NoMajorVersion,
    #[error("Manifest version must have a minor part")]
    NoMinorVersion,
    #[error("Manifest version must have a patch part")]
    NoPatchVersion,
    #[error(transparent)]
    InvalidPart(#[from] ParseIntError),
}

impl ManifestVersion {
    pub const DEFAULT: Self = Self(1, 9, 0);
    const PARTS_COUNT: u8 = 3;
    const SEPARATOR: char = '.';

    pub fn new<S: AsRef<str>>(input: S) -> Result<Self, ManifestVersionError> {
        let mut parts = input
            .as_ref()
            .splitn(Self::PARTS_COUNT as usize, Self::SEPARATOR);
        let major = parts
            .next()
            .ok_or(ManifestVersionError::NoMajorVersion)?
            .parse::<u16>()?;
        let minor = parts
            .next()
            .ok_or(ManifestVersionError::NoMinorVersion)?
            .parse::<u16>()?;
        let patch = parts
            .next()
            .ok_or(ManifestVersionError::NoPatchVersion)?
            .parse::<u16>()?;
        Ok(Self(major, minor, patch))
    }

    pub const fn const_display_fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writec!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl Default for ManifestVersion {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl FromStr for ManifestVersion {
    type Err = ManifestVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
