use crate::prompts::prompt::RequiredPrompt;
use derive_more::{Deref, Display};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;
use versions::Versioning;

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
pub struct PackageVersion(Versioning);

impl PackageVersion {
    pub fn new(input: &str) -> Result<Self, versions::Error> {
        Ok(Self(Versioning::from_str(input)?))
    }
}

impl FromStr for PackageVersion {
    type Err = versions::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

impl RequiredPrompt for PackageVersion {
    const MESSAGE: &'static str = "Package version:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: 1.2.3");
    const PLACEHOLDER: Option<&'static str> = None;
}
