use std::str::FromStr;

use color_eyre::eyre::Error;
use derive_more::{Display, FromStr};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use versions::Version;

#[derive(Clone, Debug, Default, Display, Eq, FromStr, Hash, Ord, PartialEq, PartialOrd)]
pub struct MinimumOSVersion(Version);

impl MinimumOSVersion {
    pub fn new(input: &str) -> color_eyre::Result<Self> {
        Ok(Self(Version::from_str(input).map_err(Error::msg)?))
    }

    pub fn removable() -> Self {
        Self::new("10.0.0.0").unwrap()
    }
}

impl Serialize for MinimumOSVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for MinimumOSVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
