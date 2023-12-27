use crate::prompts::prompt::RequiredPrompt;
use color_eyre::eyre::{Error, Result};
use derive_more::{Deref, Display};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use versions::Versioning;

#[derive(Clone, Default, Deref, Display, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageVersion(Versioning);

impl PackageVersion {
    pub fn new(input: &str) -> Result<Self> {
        Ok(Self(Versioning::from_str(input).map_err(Error::msg)?))
    }
}

impl Serialize for PackageVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for PackageVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl FromStr for PackageVersion {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        Self::new(input)
    }
}

impl RequiredPrompt for PackageVersion {
    const MESSAGE: &'static str = "Package version:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: 1.2.3");
    const PLACEHOLDER: Option<&'static str> = None;
}
