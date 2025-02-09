use crate::prompts::text::TextPrompt;
use crate::prompts::Prompt;
use derive_more::{AsRef, Deref, Display, FromStr};
use icu_locid::{langid, LanguageIdentifier};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(
    AsRef, Clone, Debug, Deref, Display, Deserialize, Serialize, FromStr, Eq, PartialEq, Hash,
)]
pub struct LanguageTag(LanguageIdentifier);

impl Default for LanguageTag {
    fn default() -> Self {
        Self(langid!("en-US"))
    }
}

impl Ord for LanguageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0)
    }
}

impl PartialOrd for LanguageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Prompt for LanguageTag {
    const MESSAGE: &'static str = "Package locale:";
}

impl TextPrompt for LanguageTag {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: en-US");
    const PLACEHOLDER: Option<&'static str> = None;
}
