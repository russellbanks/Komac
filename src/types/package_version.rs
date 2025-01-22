use crate::prompts::prompt::Prompt;
use crate::types::version::Version;
use derive_more::{Deref, Display};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

#[derive(
    SerializeDisplay,
    DeserializeFromStr,
    Clone,
    Default,
    Deref,
    Display,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct PackageVersion(Version);

impl PackageVersion {
    pub fn new(input: &str) -> Self {
        Self(Version::new(input))
    }
}

impl FromStr for PackageVersion {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(input))
    }
}

impl Prompt for PackageVersion {
    const MESSAGE: &'static str = "Package version:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: 1.2.3");
    const PLACEHOLDER: Option<&'static str> = None;
}
