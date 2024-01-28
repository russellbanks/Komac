use crate::prompts::prompt::RequiredPrompt;
use derive_more::{AsRef, Deref, Display, FromStr};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    AsRef,
    Clone,
    Debug,
    Deref,
    Display,
    Deserialize,
    Serialize,
    FromStr,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
pub struct LanguageTag(oxilangtag::LanguageTag<String>);

impl Default for LanguageTag {
    fn default() -> Self {
        Self::from_str("en-US").unwrap()
    }
}

impl RequiredPrompt for LanguageTag {
    const MESSAGE: &'static str = "Package locale:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: en-US");
    const PLACEHOLDER: Option<&'static str> = None;
}
