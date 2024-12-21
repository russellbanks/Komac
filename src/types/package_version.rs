use crate::prompts::prompt::Prompt;
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

impl Prompt for PackageVersion {
    const MESSAGE: &'static str = "软件包版本:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: 1.2.3 (应与控制面板或设置中的版本号相匹配)"); // 温馨提示，欸嘿
    const PLACEHOLDER: Option<&'static str> = None;
}
