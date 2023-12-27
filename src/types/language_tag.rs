use crate::prompts::prompt::RequiredPrompt;
use derive_more::{AsRef, Deref, Display, FromStr};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(
    AsRef, Clone, Debug, Deref, Display, Deserialize, Serialize, FromStr, Eq, PartialEq, Hash,
)]
pub struct LanguageTag(language_tags::LanguageTag);

impl PartialOrd for LanguageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LanguageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Default for LanguageTag {
    fn default() -> Self {
        Self::from_str("en-US").unwrap()
    }
}

impl RequiredPrompt for LanguageTag {
    const MESSAGE: &'static str = "Package locale";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: en-US");
    const PLACEHOLDER: Option<&'static str> = None;
}
